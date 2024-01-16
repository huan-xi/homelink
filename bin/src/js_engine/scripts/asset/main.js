// import env from "ext:deno_env/01_env.js";
// console.log("fetch", await fetch("https://www.baidu.com"));
console.log("main module exec")
try {
    console.log("js env", env.info)
    let ch = env.get_main_channel();
    let code = await ch.listen();
    console.error(`exit main module code: ${code}`);
} catch (e) {
    console.error("e", e)
}