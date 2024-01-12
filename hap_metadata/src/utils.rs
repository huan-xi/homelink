pub fn pascal_case(param: &str) -> String {
    // let param = h.param(0).unwrap().value().as_str().unwrap().to_owned();
    let param = param.replace("-", " ");
    let name = param
        .to_lowercase()
        .split(" ")
        .into_iter()
        .map(|word| {
            let mut c = word.chars().collect::<Vec<char>>();
            c[0] = c[0].to_uppercase().nth(0).unwrap();
            c.into_iter().collect::<String>()
        })
        .collect::<String>();
    name.replace(" ", "").replace(".", "_")
}