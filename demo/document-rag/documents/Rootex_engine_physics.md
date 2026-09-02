<!-- Source: https://rootex.readthedocs.io/en/latest/engine/physics.html -->

Physics — Rootex documentation
- Physics
# Physics
Physics in Rootex has been implemented using the Bullet Collision Detection and Physics library ([https://pybullet.org/Bullet/BulletFull/index.html](https://pybullet.org/Bullet/BulletFull/index.html))
Rootex uses the concept of colliders, adopted from the Bullet Physics library, to represent objects responding to physics and being controlled by it.
[Class PhysicsSystem](https://rootex.readthedocs.io/en/latest/api/class_physics_system.html#class-physicssystem) is responsible for providing physics to the Rootex engine. The physics system uses [Class PhysicsColliderComponent](https://rootex.readthedocs.io/en/latest/api/class_physics_collider_component.html#class-physicscollidercomponent) instances to perform physics based calculations on them. Rootex engine currently supports all the collision shapes provides by Bullet. There are also extra features available like ray casting a ray into the world and reporting the colliders that the ray touched.
The Rootex Editor also helps in visualizing the collider shapes enabled by viewing the collider component in the Inspector.
