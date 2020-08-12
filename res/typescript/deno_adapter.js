require(["typescript"], (ts) => {
  // console.log("\n\n***************************\n\n")
  // console.log(Object.keys(ts).sort().join("\n"))
  // console.log("\n\n***************************\n\n")
  var ops = Deno.core.ops();

  const bytes = Deno.core.dispatch(ops["get_string"]);
  const source = utf8ArrayToString(bytes);

  const raw_output = ts.transpileModule(source, { compilerOptions: {} });

  const res = Deno.core.dispatch(
    ops["return_string"],
    to_uint8array(raw_output.outputText)
  );
});
