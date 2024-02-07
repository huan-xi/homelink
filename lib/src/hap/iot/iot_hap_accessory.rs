use std::collections::HashMap;
use std::sync::Arc;
use impl_new::New;

use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;

use hap::accessory::HapAccessory;
use hap::characteristic::delegate::{CharReadsDelegate, CharUpdateDelegate, Cid, OnReadsFn, OnRwFn, OnUpdatesFn};
use hap::HapType;
use hap::service::HapService;

use crate::hap::models::{AccessoryModel};

/// 一个设备可能存在多个配件
/// 一个配件多个服务，一个服务多个特征值


pub struct TagsIdMap {
    map: HashMap<String, Vec<u64>>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, New)]
pub struct CharId {
    aid: u64,
    iid: u64,
}

/// 特征值处理器
pub struct CharsHandler {
    /// 一个函数可以处理多个特征值
    reads_map: HashMap<Vec<u64>, Arc<OnReadsFn>>,
    updates_map: HashMap<Vec<u64>, Arc<OnUpdatesFn>>,
}

pub struct IotHapAccessory {
    /// ID of the Switch accessory.
    id: u64,
    // pub device: Arc<dyn MiotSpecDevice + Send + Sync>,
    /// Accessory Information service.
    // pub accessory_information: AccessoryInformationService,
    /// Switch service.
    pub services: HashMap<u64, Box<dyn HapService>>,
    pub tag_ids_map: HashMap<String, Vec<u64>>,

    pub model_ext: Option<AccessoryModel>,
    reads_map: HashMap<Vec<u64>, Arc<OnReadsFn>>,
    updates_map: HashMap<Vec<u64>, Arc<OnUpdatesFn>>,
}

impl IotHapAccessory {
    pub fn new(id: u64, scv_list: Vec<Box<dyn HapService>>, model_ext: Option<AccessoryModel>) -> Self {
        let mut services = HashMap::new();
        scv_list.into_iter().for_each(|i| {
            services.insert(i.get_id(), i);
        });

        Self {
            id,
            services,
            tag_ids_map: Default::default(),
            model_ext,
            reads_map: Default::default(),
            updates_map: Default::default(),
        }
    }
    pub fn on_read() {}
}

impl HapAccessory for IotHapAccessory {
    fn get_id(&self) -> u64 {
        self.id
    }

    fn set_id(&mut self, id: u64) {
        self.id = id
    }

    fn get_service(&self, hap_type: HapType) -> Option<&dyn HapService> {
        todo!()
    }
    fn get_mut_service(&mut self, hap_type: HapType) -> Option<&mut dyn HapService> {
        todo!()
    }
    fn get_services(&self) -> Vec<&dyn HapService> {
        self.services.iter().map(|i| i.1.as_ref()).collect()
    }

    fn get_mut_services<'a>(&'a mut self) -> Vec<&'a mut dyn HapService> {
        self.services.values_mut().map(|i| i.as_mut() as (&mut dyn HapService )).collect()
    }

    fn push_service(&mut self, tag: Option<String>, service: Box<dyn HapService>) {
        if let Some(tag) = tag {
            self.tag_ids_map.entry(tag)
                .or_insert(vec![])
                .push(service.get_id())
        };
        self.services.insert(service.get_id(), service);
    }

    fn get_mut_services_by_tag(&mut self, tag: &str) -> Vec<&mut dyn HapService> {
        let ids = self.tag_ids_map.get(tag);
        if let Some(ids) = ids {
            return self.services
                .values_mut()
                .filter(|i| ids.contains(&i.get_id()))
                .map(|i| i.as_mut() as (&mut dyn HapService )).collect();
        }
        vec![]
    }
    fn get_mut_service_by_id(&mut self, id: u64) -> Option<&mut dyn HapService> {
        self.services.get_mut(&id).map(|i| i.as_mut() as &mut dyn HapService)
    }
    fn get_on_read_delegate(&self) -> Option<Box<dyn CharReadsDelegate>> {
        self.model_ext
            .as_ref()
            .and_then(|i| i.get_on_read_delegate())
    }

    fn get_on_updates_delegate(&self) -> Option<Box<dyn CharUpdateDelegate>>  {
        self.model_ext
            .as_ref()
            .and_then(|i| i.get_on_update_delegate())
    }

    /* fn get_on_reads_fn(&self) -> Option<OnReadsFn> {
         // let func_map = self.func_map.clone();
         let func_map_c = self.reads_map.clone();
         let ext = self.model_ext.clone();
         Some(Box::new(move |mut params| {
             let func_map = func_map_c.clone();
             let ext = ext.clone();
             Box::pin(async move {
                 //value中匹配iid
                 let mut result = vec![];
                 for (ids, func) in func_map.iter() {
                     let mut func_params = vec![];
                     for i in 0..params.len() {
                         let param = params.get(i).unwrap();
                         if ids.contains(&param.cid) {
                             func_params.push(params.remove(i));
                         }
                     }
                     if !func_params.is_empty() {
                         result.extend(func(func_params).await?);
                     }
                 }
                 if !params.is_empty() {
                     if let Some(ext) = ext {
                         result.extend(ext.model_ext.read_chars_value(params).await?);
                     } else {
                         info!("未处理读取参数:{:?}",params);
                     };
                 };
                 //读取函数
                 Ok(result)
             })
         }))
     }
     fn get_on_updates_fn(&self) -> Option<OnUpdatesFn> {
         let func_map_c = self.updates_map.clone();
         let ext = self.model_ext.clone();
         Some(Box::new(move |mut params| {
             let func_map = func_map_c.clone();
             let ext = ext.clone();
             Box::pin(async move {
                 //value中匹配iid
                 let mut result = vec![];
                 for (ids, func) in func_map.iter() {
                     let mut func_params = vec![];
                     for i in 0..params.len() {
                         let param = params.get(i).unwrap();
                         if ids.contains(&param.cid) {
                             func_params.push(params.remove(i));
                         }
                     }
                     if !func_params.is_empty() {
                         result.extend(func(func_params).await?);
                     }
                 }
                 if !params.is_empty() {
                     if let Some(ext) = ext {
                         result.extend(ext.model_ext.update_chars_value(params).await?);
                     } else {
                         info!("未处理更新参数:{:?}",params);
                     }
                 };
                 Ok(result)
             })
         }))
     }*/
}

impl IotHapAccessory {
    /*fn get_func<P, R,F>(&self, func_map: HashMap<Vec<u64>, Arc<OnRwFn<P, R>>>,
                        ext_func: Arc<OnRwFn<P, R>>) -> Option<OnRwFn<P, R>>
        where P: Cid {
        let func_map_c = func_map.clone();
        let ext = self.model_ext.clone();
        Some(Box::new(move |mut params| {
            let func_map = func_map_c.clone();
            let ext = ext.clone();
            Box::pin(async move {
                //value中匹配iid
                let mut result = vec![];
                for (ids, func) in func_map.iter() {
                    let mut func_params = vec![];
                    for i in 0..params.len() {
                        let param = params.get(i).unwrap();
                        if ids.contains(&param.get_cid()) {
                            func_params.push(params.remove(i));
                        }
                    }
                    if !func_params.is_empty() {
                        result.extend(func(func_params).await?);
                    }
                }
                if !params.is_empty() {
                    if let Some(ext) = ext {
                        // ext_func()
                        result.extend(ext.model_ext.update_chars_value(params).await?);
                    };
                };
                Ok(result)
            })
        }))
}*/
}

impl Serialize for IotHapAccessory {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("HapAccessory", 2)?;
        state.serialize_field("aid", &self.get_id())?;
        state.serialize_field("services", &self.get_services())?;
        state.end()
    }
}
