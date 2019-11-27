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

#[test]
pub fn test_duktape() {
    let data = read_file("tests/programs/sample.js");
    let code = read_file("tests/programs/duktape");
    let script_args = data.pack();
    let (code_hash, code_dep) = create_mock_cell_dep(code, None, None);

    let script = Script::new_builder()
        .code_hash(code_hash.clone())
        .hash_type(ScriptHashType::Data.into())
        .args(script_args)
        .build();
    let script_hash = script.clone().calc_script_hash();
    let (_, input_dep) = create_mock_cell_dep(Bytes::from(""), None, Some(script.clone()));
    dbg!(&script_hash);
    let cell_input = CellInput::new_builder()
        .previous_output(input_dep.cell_dep.out_point())
        .build();
    let cell_output = CellOutput::new_builder()
        .type_(Some(script.clone()).pack())
        .build();
    let cell_output2 = CellOutput::new_builder().build();
    let transaction = TransactionBuilder::default()
        .input(cell_input.clone())
        .output(cell_output)
        .output(cell_output2)
        .output_data(packed::Bytes::default())
        .output_data(packed::Bytes::default())
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
        None,
    );
    // assert_eq!(result.unwrap(), 58897);
    dbg!(&result);
    if let Err(e) = result {
        println!("{}", e);
    }
}
