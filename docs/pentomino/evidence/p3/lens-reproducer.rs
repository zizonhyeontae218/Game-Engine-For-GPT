#[path="/workspace/GE4G/crates/ge4g-pentomino-view/examples/support/mod.rs"] mod support;
use ge4g_pentomino::{PluginId,Bindings};
use ge4g_pentomino::p2::CoreHost;
use ge4g_pentomino_view::*;
fn main(){
 let owner=PluginId::new("audit.source").unwrap(); let source=support::Source{owner:owner.clone(),count:0};let schema=source.schema();
 let mut c=CoreHost::new(1,"audit.lens").unwrap();c.install(Box::new(source),Bindings::new()).unwrap();
 let read=c.select(&support::selection([owner.clone()])).unwrap();
 let mut h=ViewHost::new("audit.lens").unwrap();let v=ViewId::new("audit.view").unwrap();let id=CameraId::new("audit.camera").unwrap();
 h.install(Box::new(TopDownFormat),ViewConfig{id:v.clone(),scene:read.scenes[0].reference.clone(),binding:RecordBinding{owner,schema,entity_field:"entity".into(),x_field:"x".into(),y_field:"y".into(),z_field:Some("z".into()),units_per_world:1.,representation:Representation::Sprite{asset:"a".into(),size:[1.,1.]}},policy:ViewPolicy::top_down()},&read).unwrap();
 let t=CameraTarget{pose:CameraPose::default(),view_transform:ViewTransform::default(),projection:Projection::Orthographic{half_height:1.,near:59368539.700403444,far:59368539.70140345,focus_distance:1.}};
 h.add_camera(CameraConfig::new(id.clone(),v,Viewport{x:0.,y:0.,width:1.,height:1.},t.clone()),&read).unwrap();h.activate(&id).unwrap();
 let mut goal=t;goal.projection=Projection::Orthographic{half_height:1.,near:55827401.02394892,far:55827401.024948925,focus_distance:1.};
 for tick in 1..=4096 {let before=h.save().unwrap();let r=h.update(&read,ViewInput{target_tick:tick,changes:if tick==1 {vec![CameraChange{camera:id.clone(),target:goal.clone(),transition:Some(TransitionMode::Smooth{duration_ticks:4096})}]}else{vec![]}});if let Err(e)=r {println!("VALID_ENDPOINT_FAILURE tick={tick} code={:?} rollback={}",e.code(),before==h.save().unwrap());return;}}
 println!("PASS");
}
