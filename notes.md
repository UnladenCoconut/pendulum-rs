indexed mesh/es

scene
    objects
        meshes

n objects, with object transform in world space

m parts in object (4 for pendulum), made of o meshes

so indexed meshes, per-part transforms in object space 


- vertex and indicies buffers
- meshes denoted by contiguous? index range. indexes per mesh have to be offset by their position?
so mesh id -> vertex id offset 


each object has o unique mesh transforms for each part
- object buffer of object transforms
- mesh instance transform buffer - object id transform idx (for object buffer ), object-space part transform
 