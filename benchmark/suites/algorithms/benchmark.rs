fn main() {
    let a: i32 = 12;
    let b: i32 = 8;
    let c: i32 = 5;
    let d: i32 = 3;

    let result: i32 = (a * b) + (c * d) - (a * c) + (b * d);

    println!("{}", result);
}