var td = new TextDecoder()

function print(obj, name) {
    var obj_str = Duktape.enc('jc', obj)
    CKB.debug(name || 'obj', ':', obj_str)
}

function objectEquals(x, y) {
    if (x === null || x === undefined || y === null || y === undefined) { return x === y; }
    // after this just checking type of one would be enough
    if (x.constructor !== y.constructor) { return false; }
    // if they are functions, they should exactly refer to same one (because of closures)
    if (x instanceof Function) { return x === y; }
    // if they are regexps, they should exactly refer to same one (it is hard to better equality check on current ES)
    if (x instanceof RegExp) { return x === y; }
    if (x === y || x.valueOf() === y.valueOf()) { return true; }
    if (Array.isArray(x) && x.length !== y.length) { return false; }

    // if they are dates, they must had equal valueOf
    if (x instanceof Date) { return false; }

    // if they are strictly equal, they both need to be object at least
    if (!(x instanceof Object)) { return false; }
    if (!(y instanceof Object)) { return false; }

    // recursive object equality check
    var p = Object.keys(x);
    return Object.keys(y).every(function (i) { return p.indexOf(i) !== -1; }) &&
        p.every(function (i) { return objectEquals(x[i], y[i]); });
}

function get_cell_num(type) {
    var i = 0
    while (true) {
        var ret = CKB.load_cell_data(0, i, type)
        if (typeof ret === "number") {
            return i
        }
        i += 1
    }
}

function assert(condition, message) {
    message = message || "assert failed"
    if (!condition) {
        throw message
    }
}

/**
 * check current tx type
 *
 * return: 'init' | 'call'
 */
function check_tx_type() {
    var group_output_num = get_cell_num(CKB.SOURCE.GROUP_OUTPUT)
    if (group_output_num !== 1) {
        throw "there must be exactly 1 output with udt script!"
    }
    var group_input_num = get_cell_num(CKB.SOURCE.GROUP_INPUT)
    if (group_input_num === 0) {
        return 'init'
    } else if (group_input_num === 1) {
        return 'call'
    } else {
        throw "there must be 0 or 1 input with udt script!"
    }
}

function check_balance(balances, total_supply) {
    for (var k in balances) {
        assert(Number.isInteger(balances[k]), 'balance must be integer')
        if (balances[k] < 0) {
            throw "balance can not be negtive"
        }
        total_supply -= balances[k]
        if (total_supply < 0) {
            throw "sum of balances is greater than total supply"
        }
    }
    if (total_supply !== 0) {
        throw "sum of balances is not equal to total supply"
    }
}

function verify_init() {
    // todo: change json serde method to a better one
    var group_output = CKB.load_cell_data(0, 0, CKB.SOURCE.GROUP_OUTPUT)
    var group_output_str = td.decode(group_output)
    var group_output_obj = Duktape.dec('jc', group_output_str)

    var input = CKB.load_input(0, 0, CKB.SOURCE.INPUT)
    /**
     * contract_id is the outpoint of the first input.
     *
     * nobody can create the contract with the same contract_id anymore because the
     * chain ensures that the cell can not be consumed again.
     */
    var contract_id  = Duktape.enc('hex', input).slice(16)
    assert(group_output_obj.contract_id == contract_id, "contract_id invalid")
    assert(Number.isInteger(group_output_obj.total_supply), 'total_supply must be integer')
    assert(group_output_obj.total_supply > 0, "total supply invalid")
    check_balance(group_output_obj.balances, group_output_obj.total_supply)
}

function verify_action(action, data) {
    // TODO: replace code below with real signature check method
    assert(action['signature'] == 'magic', 'invalid action signature')
    // TODO: find a better way to avoid replay attack
    var current_nonce = data.nonces[action['from']] || 0
    assert(current_nonce == action['nonce'], 'nonce not match')
    data.nonces[action['from']] = current_nonce + 1
}

function execute(action, data) {
    verify_action(action, data)
    if (action['action'] == 'transfer') {
        assert(Number.isInteger(action['params']['amount']), 'transfer amount must be an integer')
        assert(action['params']['amount'] > 0, 'transfer amount must be positive')
        var from_balance = data.balances[action['from']] || 0
        assert(from_balance > action['params']['amount'], 'from balance not enough')
        data.balances[action['from']] -= action['params']['amount']
        data.balances[action['params']['to']] = data.balances[action['params']['to']] || 0 + action['params']['amount']
        return data
    }

}

function verify_call() {
    var group_input = CKB.load_cell_data(0, 0, CKB.SOURCE.GROUP_INPUT)
    var group_input_str = td.decode(group_input)
    var group_input_obj = Duktape.dec('jc', group_input_str)

    var group_output = CKB.load_cell_data(0, 0, CKB.SOURCE.GROUP_OUTPUT)
    var group_output_str = td.decode(group_output)
    var group_output_obj = Duktape.dec('jc', group_output_str)

    print(group_input_obj, 'group_input_obj')
    print(group_output_obj, 'group_output_obj')

    var action_index = 1
    var data = group_input_obj
    while (true) {
        var ret = CKB.load_cell_data(0, action_index, CKB.SOURCE.INPUT)
        if (typeof ret === "number") {
            break
        }
        // CKB.debug(td.decode(ret))
        var actions = Duktape.dec('jc', td.decode(ret))
        var actionsLen = actions.length;
        // print(actions, 'actions')
        for(var i = 0; i< actionsLen; i++) {
            var action = actions[i]
            print(action, 'action')
            data = execute(action, data)
            print(data, 'data')
        }
        action_index += 1
    }
    assert(objectEquals(group_output_obj, data), 'output not valid')
}

function main() {
    CKB.debug("start udt script")
    var tx_type = check_tx_type()
    CKB.debug("tx_type:", tx_type)
    if (tx_type == 'init') {
        verify_init()
    } else {
        verify_call()
    }
}

main()