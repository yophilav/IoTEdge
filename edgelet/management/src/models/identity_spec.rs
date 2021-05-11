/* 
 * IoT Edge Management API
 *
 * No description provided (generated by Swagger Codegen https://github.com/swagger-api/swagger-codegen)
 *
 * OpenAPI spec version: 2020-07-07
 * 
 * Generated by: https://github.com/swagger-api/swagger-codegen.git
 */


#[allow(unused_imports)]
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct IdentitySpec {
  #[serde(rename = "moduleId")]
  module_id: String,
  #[serde(rename = "managedBy")]
  managed_by: Option<String>
}

impl IdentitySpec {
  pub fn new(module_id: String) -> IdentitySpec {
    IdentitySpec {
      module_id: module_id,
      managed_by: None
    }
  }

  pub fn set_module_id(&mut self, module_id: String) {
    self.module_id = module_id;
  }

  pub fn with_module_id(mut self, module_id: String) -> IdentitySpec {
    self.module_id = module_id;
    self
  }

  pub fn module_id(&self) -> &String {
    &self.module_id
  }


  pub fn set_managed_by(&mut self, managed_by: String) {
    self.managed_by = Some(managed_by);
  }

  pub fn with_managed_by(mut self, managed_by: String) -> IdentitySpec {
    self.managed_by = Some(managed_by);
    self
  }

  pub fn managed_by(&self) -> Option<&String> {
    self.managed_by.as_ref()
  }

  pub fn reset_managed_by(&mut self) {
    self.managed_by = None;
  }

}



