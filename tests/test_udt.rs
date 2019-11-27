use ckb_script::ScriptGroupType;
use ckb_sdk_types::transaction::{MockCellDep, MockInfo, MockInput, MockTransaction};
use ckb_standalone_debugger::run;
use ckb_types::{
    bytes::Bytes,
    core::{Capacity, DepType, ScriptHashType, TransactionBuilder},
    packed::{self, Byte32, CellDep, CellInput, CellOutput, OutPoint, Script, ScriptOpt},
    prelude::*,
};
use std::fs::File;
use std::io::Read;

fn read_file(name: &str) -> Bytes {
    let mut file = File::open(name).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    buffer.into()
}

fn create_mock_cell_dep(
    data: Bytes,
    lock: Option<Script>,
    type_: Option<Script>,
) -> (Byte32, MockCellDep) {
    let hash = CellOutput::calc_data_hash(&data);
    let hash2 = CellOutput::calc_data_hash(hash.as_slice());
    let out_point = OutPoint::new_builder().tx_hash(hash2).build();
    let cell_dep = CellDep::new_builder()
        .out_point(out_point)
        .dep_type(DepType::Code.into())
        .build();
    let cell_output = CellOutput::new_builder()
        .capacity(Capacity::bytes(data.len() + 200).unwrap().pack())
        .lock(lock.unwrap_or_else(Script::default))
        .type_(ScriptOpt::new_builder().set(type_).build())
        .build();
    (
        hash,
        MockCellDep {
            cell_dep,
            output: cell_output,
            data,
        },
    )
}

fn ckb_debug_printer(_script_hash: &Byte32, debug_info: &str) {
    // dbg!(script_hash, debug_info);
    println!("[ckb debug] {}", debug_info);
}

#[test]
pub fn test_udt_init() {
    let data = read_file("tests/programs/udt.js");
    let code = read_file("tests/programs/duktape");
    let script_args = data.pack();
    let (code_hash, code_dep) = create_mock_cell_dep(code, None, None);

    let udt_script = Script::new_builder()
        .code_hash(code_hash.clone())
        .hash_type(ScriptHashType::Data.into())
        .args(script_args)
        .build();
    let script_hash = udt_script.clone().calc_script_hash();
    // dbg!(&script_hash);
    let (_, input_dep) = create_mock_cell_dep(Bytes::from(""), None, None);
    // dbg!(&input_dep.cell_dep.out_point());
    let cell_input = CellInput::new_builder()
        .previous_output(input_dep.cell_dep.out_point())
        .build();
    let cell_output = CellOutput::new_builder()
        .type_(Some(udt_script.clone()).pack())
        .build();
    let out_data = format!(
        r#"
{{
    "contract_id": "{:x}",
    "total_supply": 100000000,
    "balances": {{"addr1": 100000000}}
}}
    "#,
        input_dep.cell_dep.out_point()
    );
    let transaction = TransactionBuilder::default()
        .input(cell_input.clone())
        .output(cell_output)
        .output_data(Bytes::from(out_data).pack())
        .witness(packed::Bytes::default())
        .cell_dep(code_dep.cell_dep.clone())
        .build();
    let mock_input = MockInput {
        input: cell_input,
        output: input_dep.output,
        data: input_dep.data,
    };
    let mock_info = MockInfo {
        inputs: vec![mock_input],
        cell_deps: vec![code_dep],
        header_deps: vec![],
    };
    let mock_transaction = MockTransaction {
        mock_info,
        tx: transaction.data(),
    };

    let result = run(
        &mock_transaction,
        &ScriptGroupType::Type,
        &script_hash,
        20_000_000,
        Some(Box::new(ckb_debug_printer)),
    );
    // assert_eq!(result.unwrap(), 58897);
    dbg!(&result);
    assert_eq!(result.is_ok(), true);
}

#[test]
pub fn test_udt_transfer() {
    let data = read_file("tests/programs/udt.js");
    let code = read_file("tests/programs/duktape");
    let script_args = data.pack();
    let (code_hash, code_dep) = create_mock_cell_dep(code, None, None);

    let udt_script = Script::new_builder()
        .code_hash(code_hash.clone())
        .hash_type(ScriptHashType::Data.into())
        .args(script_args)
        .build();
    let script_hash = udt_script.clone().calc_script_hash();
    // dbg!(&script_hash);
    let in_data = r#"
{
    "contract_id": "mock_contract_id",
    "total_supply": 100000000,
    "balances": {"addr1": 100000000},
    "nonces": {}
}
    "#;
    let (_, input_dep) = create_mock_cell_dep(Bytes::from(in_data), None, Some(udt_script.clone()));
    // dbg!(&input_dep.cell_dep.out_point());
    let cell_input = CellInput::new_builder()
        .previous_output(input_dep.cell_dep.out_point())
        .build();

    let action_script_file = read_file("tests/programs/action.js");
    // replace contract_script_hash in the script with current one
    let action_script_file: Bytes = std::str::from_utf8(action_script_file.as_ref())
        .unwrap()
        .replace(
            "contract_script_hash_holder",
            &format!("{:x}", &script_hash),
        )
        .into();
    let action_script = Script::new_builder()
        .code_hash(code_hash.clone())
        .hash_type(ScriptHashType::Data.into())
        .args(action_script_file.pack())
        .build();
    let action_script_hash = action_script.clone().calc_script_hash();
    // dbg!(&action_script_hash);

    let action_data = r#"
[
    {
        "contract_id": "mock_contract_id",
        "action": "transfer",
        "from": "addr1",
        "params": {
            "to": "addr2",
            "amount": 100
        },
        "nonce": 0,
        "signature": "magic"
    },
    {
        "contract_id": "mock_contract_id",
        "action": "transfer",
        "from": "addr1",
        "params": {
            "to": "addr3",
            "amount": 200
        },
        "nonce": 1,
        "signature": "magic"
    }
]
    "#;
    let (_, action_input_dep) =
        create_mock_cell_dep(Bytes::from(action_data), None, Some(action_script.clone()));
    let action_cell_input = CellInput::new_builder()
        .previous_output(action_input_dep.cell_dep.out_point())
        .build();

    let cell_output = CellOutput::new_builder()
        .type_(Some(udt_script.clone()).pack())
        .build();
    let out_data = r#"
{
    "contract_id": "mock_contract_id",
    "total_supply": 100000000,
    "balances": {"addr3": 200, "addr2": 100, "addr1": 99999700},
    "nonces": {"addr1": 2}
}
    "#;
    let transaction = TransactionBuilder::default()
        .input(cell_input.clone())
        .input(action_cell_input.clone())
        .output(cell_output)
        .output_data(Bytes::from(out_data).pack())
        .witness(packed::Bytes::default())
        .cell_dep(code_dep.cell_dep.clone())
        .build();
    let mock_input = MockInput {
        input: cell_input,
        output: input_dep.output,
        data: input_dep.data,
    };
    let mock_input2 = MockInput {
        input: action_cell_input,
        output: action_input_dep.output,
        data: action_input_dep.data,
    };
    let mock_info = MockInfo {
        inputs: vec![mock_input, mock_input2],
        cell_deps: vec![code_dep],
        header_deps: vec![],
    };
    let mock_transaction = MockTransaction {
        mock_info,
        tx: transaction.data(),
    };

    let udt_result = run(
        &mock_transaction,
        &ScriptGroupType::Type,
        &script_hash,
        20_000_000,
        Some(Box::new(ckb_debug_printer)),
    );
    dbg!(&udt_result);
    assert_eq!(udt_result.is_ok(), true);

    let action_result = run(
        &mock_transaction,
        &ScriptGroupType::Type,
        &action_script_hash,
        20_000_000,
        Some(Box::new(ckb_debug_printer)),
    );
    dbg!(&action_result);
    assert_eq!(action_result.is_ok(), true);
}
