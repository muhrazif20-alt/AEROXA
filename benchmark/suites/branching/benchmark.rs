fn main() {
    let score: i32 = 85;
    let attendance: i32 = 90;

    if score >= 90 {
        println!("Excellent");
    } else if score >= 75 {
        println!("Good");
    } else {
        println!("Needs Improvement");
    }

    if attendance >= 80 {
        println!("Attendance Passed");
    } else {
        println!("Attendance Failed");
    }
}