var ops = Deno.core.ops();
var first = Deno.core.dispatch(ops["get_string"]);

var first_string = utf8ArrayToString(first);

var second_string = first_string.toUpperCase();

// setTimeout(() => {
//   console.log("Mark")
// }, 10000);

var res = Deno.core.dispatch(ops["return_string"], to_uint8array(second_string));
