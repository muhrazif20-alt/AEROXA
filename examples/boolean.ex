let is_active:Bool = true;
let is_admin:Bool =false;

@condition (is_active && !is_admin){
    let status:String = "Active User";
}