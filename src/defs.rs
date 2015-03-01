use std::collections::{HashMap, HashSet};
use std::ops::Add;
use std::iter::range_step;
use std::mem;

use libc::{c_void, malloc, size_t, free};

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct ObjectLocation(usize);

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct PointerOffset(usize);

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct ClassId(usize);

pub struct Class {
    id: ClassId,
    size: usize,
    ptr_locations: Vec<PointerOffset>
}

pub struct ClassBuilder {
    id: usize,
    size: Option<usize>,
    ptrs: Vec<PointerOffset>
}

pub struct Gc {
    classes: HashMap<ClassId, Class>,
    allocated_objs: HashMap<ObjectLocation, ClassId>,
    current_build: Option<ClassBuilder>
}

impl Gc {
    pub fn new() -> Gc {
        Gc {
            classes: HashMap::new(),
            allocated_objs: HashMap::new(),
            current_build: None
        }
    }

    pub fn add_class(&mut self, id: usize) {
        match self.current_build.take().map(|x| x.build()) {
            Some(Some(class)) => {
                self.classes.insert(class.id, class);
            }
            Some(None) => {
                panic!("Class before {} did not get a size!", id)
            }
            None => {}
        }

        self.current_build = Some(ClassBuilder::new(id));
    }

    pub fn get_class_builder(&mut self) -> Option<&mut ClassBuilder> {
        self.current_build.as_mut()
    }

    pub fn finish_building(&mut self) {
        self.add_class(0); // HACK HACK HACK
        self.current_build = None;
    }

    pub fn alloc(&mut self, id: usize, _canary: usize) -> *mut c_void {
        let class_id = ClassId(id);
        let class = self.classes.get(&class_id)
                        .expect("Tried to alloc with unknown id");

        let ptr = unsafe { malloc(class.size as u64) as *mut c_void };
        self.allocated_objs.insert(ObjectLocation(ptr as usize), class_id);
        ptr
    }

    fn class_for(&self, ptr: ObjectLocation) -> &Class {
        let id = self.allocated_objs.get(&ptr).expect("Gc is not tracking the pointer.");
        self.classes.get(&id).as_ref().expect("No class for classId.")
    }

    fn mark_and_sweep(&mut self, canary: usize) {
        let mut seen = HashSet::new();
        let mut added = Vec::new();


        // Stack black magic starts here.
        let mut this_pos: usize =  0;
        let stack_ptr: *mut usize = &mut this_pos;
        this_pos = stack_ptr as usize;

        for pos in range_step(this_pos, canary, 8) {
            let possible_pointer = unsafe { mem::transmute(pos) };
            if self.allocated_objs.contains_key(&possible_pointer) {
                seen.insert(possible_pointer);
                added.push(possible_pointer);
            }
        }
        // Stack black magic ends here.

        while let Some(next) = added.pop() {
            let class = self.class_for(next);
            for &offset in &class.ptr_locations {
                seen.insert(next + offset);
                added.push(next + offset);
            }
        }

        let known = self.allocated_objs.keys().cloned().collect();
        let unreach = seen.difference(&known);

        for unr in unreach {
            unsafe {
                let ptr = mem::transmute(unr);
                free(ptr);
            }
        }
    }
}

impl Add<PointerOffset> for ObjectLocation {
    type Output = ObjectLocation;
    fn add(self, rhs: PointerOffset) -> ObjectLocation {
        let ObjectLocation(x) = self;
        let PointerOffset(y) = rhs;
        ObjectLocation(x + y)
    }
}

impl ClassBuilder {
    fn new(id: usize) -> ClassBuilder {
        ClassBuilder {
            id: id,
            size: None,
            ptrs: Vec::new()
        }
    }

    pub fn set_size(&mut self, size: usize) {
        self.size = Some(size);
    }

    pub fn add_ptr(&mut self, ptr: usize) {
        self.ptrs.push(PointerOffset(ptr));
    }

    fn build(self) -> Option<Class> {
        self.size.map(move |sz|
            Class {
                id: ClassId(self.id),
                size: sz,
                ptr_locations: self.ptrs
            }
        )
    }
}

