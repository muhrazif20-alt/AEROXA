use std::fs;
use std::path::Path;

pub struct ProjectConfig{
    pub name:String,
    pub version:String,
}
impl ProjectConfig{
    pub fn new(name:String)->Self{
        Self{
            name,
            version:"0.1.0".to_string(),
        }
    }
    pub fn save(&self)->Result<(),String>{
        let content = format!(
            "[project]\nname =\"{}\"\nversion = \"{}\"\n",
            self.name,
            self.version
        );
        let path = Path::new("aeroxa.toml");
        if path.exists(){
            return Err("aeroxa.toml already exists".to_string());
        }
        fs::write(path,content).map_err(|error|format!("Failed to create configuration:{}",error))?;
        Ok(())
    }
}