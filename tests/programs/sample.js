function print(obj, name) {
    var obj_str = Duktape.enc('jc', obj);
    CKB.debug(name || 'obj', ':', obj_str);
}

function get_cell_num(type) {
    var i = 0;
    while (true) {
        var ret =  CKB.load_cell_data(0, i, type);
        if (typeof ret === "number") {
            return i;
        }
        i += 1;
    }
}

function main() {
    CKB.debug("start main");
    var buffer = new ArrayBuffer(20);
    var ret = CKB.raw_load_cell_data(buffer, 0, 0, CKB.SOURCE.GROUP_INPUT);
    print(ret);
    print(buffer);
    var buffer = new ArrayBuffer(20);
    var ret = CKB.raw_load_cell_data(buffer, 0, 0, CKB.SOURCE.GROUP_OUTPUT);
    print(ret);
    print(buffer);

    var group_input = CKB.load_cell_data(0, 0, CKB.SOURCE.GROUP_INPUT);
    print(group_input, 'group_input');
    var group_output = CKB.load_cell_data(0, 0, CKB.SOURCE.GROUP_OUTPUT);
    print(group_output, 'group_output');

    print(get_cell_num(CKB.SOURCE.GROUP_INPUT), 'group_input_num');
    print(get_cell_num(CKB.SOURCE.GROUP_OUTPUT), 'group_output_num');
    print(get_cell_num(CKB.SOURCE.INPUT), 'input_num');
    print(get_cell_num(CKB.SOURCE.OUTPUT), 'output_num');
}

main()