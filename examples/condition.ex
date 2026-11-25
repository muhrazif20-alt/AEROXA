let score:Int32 = 90;
let attendance:Int32 =  85;
let bonus:Int32 = 1;
@condition (score >= 90 && attendance >=80){
    let result:String = "Excellent";
}
@alternative (score >= 75 || bonus == 1){
    let result:String = "Good";
}
@otherwise {
    let result:String = "Keep Learning";
}