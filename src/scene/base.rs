use std::sync::{Arc, Mutex};
use crate::{
    resource::model::Model,
    scene::{
        node::Node,
        transform::Transform
    }
};
use crate::core::{
    math::{vec3::Vec3, mat4::Mat4},
    visitor::{Visit, VisitResult, Visitor},
    pool::Handle,
};

pub struct Base {
    pub name: String,
    pub local_transform: Transform,
    pub visibility: bool,
    pub global_visibility: bool,
    pub parent: Handle<Node>,
    pub children: Vec<Handle<Node>>,
    pub global_transform: Mat4,
    /// Bone-specific matrix. Non-serializable.
    pub inv_bind_pose_transform: Mat4,
    /// A resource from which this node was instantiated from, can work in pair
    /// with `original` handle to get corresponding node from resource.
    pub resource: Option<Arc<Mutex<Model>>>,
    /// Handle to node in scene of model resource from which this node
    /// was instantiated from.
    pub original: Handle<Node>,
    /// When `true` it means that this node is instance of `resource`.
    /// More precisely - this node is root of whole descendant nodes
    /// hierarchy which was instantiated from resource.
    pub is_resource_instance: bool,
    /// Maximum amount of Some(time) that node will "live" or None
    /// if node has undefined lifetime.
    pub lifetime: Option<f32>,
}

pub trait AsBase {
    fn base(&self) -> &Base;
    fn base_mut(&mut self) -> &mut Base;
}

impl Base {
    /// Sets name of node. Can be useful to mark a node to be able to find it later on.
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_owned();
    }

    /// Returns name of node.
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns shared reference to local transform of a node, can be used to fetch
    /// some local spatial properties, such as position, rotation, scale, etc.
    pub fn get_local_transform(&self) -> &Transform {
        &self.local_transform
    }

    /// Returns mutable reference to local transform of a node, can be used to set
    /// some local spatial properties, such as position, rotation, scale, etc.
    pub fn get_local_transform_mut(&mut self) -> &mut Transform {
        &mut self.local_transform
    }

    /// Sets new local transform of a node.
    pub fn set_local_transform(&mut self, transform: Transform) {
        self.local_transform = transform;
    }

    /// Sets lifetime of node in seconds, lifetime is useful for temporary objects.
    /// Example - you firing a gun, it produces two particle systems for each shot:
    /// one for gunpowder fumes and one when bullet hits some surface. These particle
    /// systems won't last very long - usually they will disappear in 1-2 seconds
    /// but nodes will still be in scene consuming precious CPU clocks. This is where
    /// lifetimes become handy - you just set appropriate lifetime for a particle
    /// system node and it will be removed from scene when time will end. This is
    /// efficient algorithm because scene holds every object in pool and allocation
    /// or deallocation of node takes very little amount of time.
    pub fn set_lifetime(&mut self, time_seconds: f32) {
        self.lifetime = Some(time_seconds);
    }

    /// Returns current lifetime of a node. Will be None if node has undefined lifetime.
    /// For more info about lifetimes see [`set_lifetime`].
    pub fn get_lifetime(&self) -> Option<f32> {
        self.lifetime
    }

    /// Returns handle of parent node.
    pub fn get_parent(&self) -> crate::core::pool::Handle<Node> {
        self.parent
    }

    /// Returns slice of handles to children nodes. This can be used, for example, to
    /// traverse tree starting from some node.
    pub fn get_children(&self) -> &[Handle<Node>] {
        self.children.as_slice()
    }

    /// Returns global transform matrix, such matrix contains combined transformation
    /// of transforms of parent nodes. This is the final matrix that describes real
    /// location of object in the world.
    pub fn get_global_transform(&self) -> Mat4 {
        self.global_transform
    }

    /// Returns inverse of bind pose matrix. Bind pose matrix - is special matrix
    /// for bone nodes, it stores initial transform of bone node at the moment
    /// of "binding" vertices to bones.
    pub fn get_inv_bind_pose_transform(&self) -> Mat4 {
        self.inv_bind_pose_transform
    }

    pub fn is_resource_instance(&self) -> bool {
        self.is_resource_instance
    }

    /// Returns resource from which this node was instantiated from.
    pub fn get_resource(&self) -> Option<Arc<Mutex<Model>>> {
        self.resource.clone()
    }

    /// Sets local visibility of a node.
    pub fn set_visibility(&mut self, visibility: bool) {
        self.visibility = visibility;
    }

    /// Returns local visibility of a node.
    pub fn get_visibility(&self) -> bool {
        self.visibility
    }

    /// Returns combined visibility of an node. This is the final visibility of a node.
    /// Global visibility calculated using visibility of all parent nodes until root one,
    /// so if some parent node upper on tree is invisible then all its children will be
    /// invisible. It defines if object will be rendered. It is *not* the same as real
    /// visibility point of view of some camera. To check if object is visible from some
    /// camera, use appropriate method (TODO: which one?)
    pub fn get_global_visibility(&self) -> bool {
        self.global_visibility
    }

    /// Handle to node in scene of model resource from which this node
    /// was instantiated from.
    pub fn get_original_handle(&self) -> Handle<Node> {
        self.original
    }

    /// Returns position of the node in absolute coordinates.
    pub fn get_global_position(&self) -> Vec3 {
        self.global_transform.position()
    }

    /// Returns "look" vector of global transform basis, in most cases return vector
    /// will be non-normalized.
    pub fn get_look_vector(&self) -> Vec3 {
        self.global_transform.look()
    }

    /// Returns "side" vector of global transform basis, in most cases return vector
    /// will be non-normalized.
    pub fn get_side_vector(&self) -> Vec3 {
        self.global_transform.side()
    }

    /// Returns "up" vector of global transform basis, in most cases return vector
    /// will be non-normalized.
    pub fn get_up_vector(&self) -> Vec3 {
        self.global_transform.up()
    }
}

/// Shallow copy of node data.
impl Clone for Base {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            local_transform: self.local_transform.clone(),
            global_transform: self.global_transform,
            visibility: self.visibility,
            global_visibility: self.global_visibility,
            inv_bind_pose_transform: self.inv_bind_pose_transform,
            resource: self.resource.clone(),
            is_resource_instance: self.is_resource_instance,
            lifetime: self.lifetime,
            // Rest of data is *not* copied!
            ..Default::default()
        }
    }
}

impl Default for Base {
    fn default() -> Self {
        BaseBuilder::new().build()
    }
}

impl AsBase for Base {
    fn base(&self) -> &Base {
        self
    }

    fn base_mut(&mut self) -> &mut Base {
        self
    }
}

impl Visit for Base {
    fn visit(&mut self, name: &str, visitor: &mut Visitor) -> VisitResult {
        visitor.enter_region(name)?;

        self.name.visit("Name", visitor)?;
        self.local_transform.visit("Transform", visitor)?;
        self.visibility.visit("Visibility", visitor)?;
        self.parent.visit("Parent", visitor)?;
        self.children.visit("Children", visitor)?;
        self.resource.visit("Resource", visitor)?;
        self.is_resource_instance.visit("IsResourceInstance", visitor)?;
        self.lifetime.visit("Lifetime", visitor)?;

        visitor.leave_region()
    }
}

pub struct BaseBuilder {
    name: Option<String>,
    visibility: Option<bool>,
    local_transform: Option<Transform>,
    children: Option<Vec<Handle<Node>>>,
    lifetime: Option<f32>,
}

impl Default for BaseBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            visibility: None,
            local_transform: None,
            children: None,
            lifetime: None,
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_owned());
        self
    }

    pub fn with_visibility(mut self, visibility: bool) -> Self {
        self.visibility = Some(visibility);
        self
    }

    pub fn with_local_transform(mut self, transform: Transform) -> Self {
        self.local_transform = Some(transform);
        self
    }

    pub fn with_children(mut self, children: Vec<Handle<Node>>) -> Self {
        self.children = Some(children);
        self
    }

    pub fn with_lifetime(mut self, time_seconds: f32) -> Self {
        self.lifetime = Some(time_seconds);
        self
    }

    pub fn build(self) -> Base {
        Base {
            name: self.name.unwrap_or_default(),
            children: self.children.unwrap_or_default(),
            local_transform: self.local_transform.unwrap_or_else(Transform::identity),
            lifetime: self.lifetime,
            visibility: self.visibility.unwrap_or(true),
            global_visibility: true,
            parent: Handle::NONE,
            global_transform: Mat4::IDENTITY,
            inv_bind_pose_transform: Mat4::IDENTITY,
            resource: None,
            original: Handle::NONE,
            is_resource_instance: false,
        }
    }
}
