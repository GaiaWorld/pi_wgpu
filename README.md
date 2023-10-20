# run example

## 本地平台： 
执行`cargo run --example triangle`命令运行triangle example

## web平台 （https://rustwasm.github.io/docs/wasm-bindgen/contributing/testing.html）
1. 执行`wasm-pack test  --chrome --example triangle`命令，构建wasm以及测试环境，并开启服务器监听在`8000`端口 
2. 在浏览器中访问`http://127.0.0.1:8000`地址，即可运行测试

## android平台
1. 打开Linux Shell, 执行`cargo apk run --example triangle --lib`编译example为apk
2. 链接手机 在`target\debug\apk\examples`中打开cmd， 并执行`adb install triangle.apk`来安装apk

