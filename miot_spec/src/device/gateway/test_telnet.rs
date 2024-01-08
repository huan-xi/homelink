use std::time::Duration;
use base64::Engine;
use base64::engine::general_purpose;
use mini_telnet::Telnet;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[tokio::test]
pub async fn test_v1() {
    let str = "Enter 'help' for a list of built-in commands.";
    let p = "built-in commands.".to_string();
    println!("{}", str.as_bytes().ends_with(p.as_bytes()));
}

#[tokio::test]
pub async fn  test() ->anyhow::Result<()>{
    let mut telnet = Telnet::builder()
        //"BusyBox v1.22.1 (2023-03-13 11:34:20 CST) built-in shell (ash)
        .prompt("built-in commands.\n")
        .prompt("# ")
        .login_prompt("rlxlinux login: ", "Password: ")
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(10))
        .connect("192.168.68.24:23")
        .await?;
    telnet.login("admin", "").await?;
    // telnet.normal_execute()
    let output = telnet.execute("cat /data/miio/mible_local.db | base64").await?;
    // base64::decode(output)?;
    // let output = output.trim_start_matches("\"");
    let output = output.replace("\n", "");
    let bytes = general_purpose::STANDARD_NO_PAD.decode(output).unwrap();
    // 写到文件
    let mut file = File::create("/Users/huanxi/tmp/mible_local.db").await?;
    file.write_all(&bytes).await?;


    // let bytes = base64::decode(output)?;
    // let bytes = base64::decode_config(output, base64::STANDARD_NO_PAD)?;

    // Alphabet::Standard => alphabet::STANDARD,
    // Alphabet::UrlSafe => alphabet::URL_SAFE,

    // println!("{}", output);
    Ok(())
}