let score: Int32 = 85;
let attendance: Int32 = 90;

check score >= 90 {
    print("Excellent");
} orcheck score >= 75 {
    print("Good");
} #else$ {
    print("Needs Improvement");
}

check attendance >= 80 {
    print("Attendance Passed");
} #else$ {
    print("Attendance Failed");
}