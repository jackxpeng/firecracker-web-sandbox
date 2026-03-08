fn main() {
    let s = String::from("hello");
    let v = serde_json::json!({ "a": s });
    println!("{}", s);
}
