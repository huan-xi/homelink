

/// hap_platform desc 字符串转 name
pub fn pascal_case(param: &str) -> String {
    // let param = h.param(0).unwrap().value().as_str().unwrap().to_owned();
    let param = param.replace('-', " ");
    let name = param
        .to_lowercase()
        .split(' ')
        .map(|word| {
            let mut c = word.chars().collect::<Vec<char>>();
            c[0] = c[0].to_uppercase().next().unwrap();
            c.into_iter().collect::<String>()
        })
        .collect::<String>();
    name.replace(' ', "").replace('.', "_")
}


fn snake_case(param: &str) -> String {
    param
        .replace([' ', '.', '-'], "_")
        .to_lowercase()
}

#[test]
pub fn test() {
    let a = "current-temperature";
    let c = snake_case(a);
    println!("{}", c);
}