function assert(condition, message) {
    message = message || "assert failed"
    if (!condition) {
        throw message
    }
}

function print(obj, name) {
    var obj_str = Duktape.enc('jc', obj)
    CKB.debug(name || 'obj', ':', obj_str)
}

function arraysEqual(a, b) {
    if (a === b) return true;
    if (a == null || b == null) return false;
    if (a.length != b.length) return false;

    for (var i = 0; i < a.length; ++i) {
        if (a[i] !== b[i]) return false;
    }
    return true;
}

function main() {
    CKB.debug("start action script")

    var contract_script_hash = 'contract_script_hash_holder'
    var action_script_hash = CKB.load_script_hash()

    // the first input cell should be the contract cell
    var type_hash = Duktape.enc('hex', CKB.load_cell_by_field(0, 0, CKB.SOURCE.INPUT, CKB.CELL.TYPE_HASH))
    assert(type_hash == contract_script_hash, 'the type hash of first input must be udt contract script hash')

    // all other input cells should be associated action cell
    for (var input_index = 1; type_hash !== CKB.CODE.INDEX_OUT_OF_BOUND; input_index++) {
        type_hash = CKB.load_cell_by_field(0, input_index, CKB.SOURCE.INPUT, CKB.CELL.TYPE_HASH)
        assert(arraysEqual(type_hash, action_script_hash), 'all input cells except the first one should be action cell, whose type script hash is the same with this one')
    }
}

main()
