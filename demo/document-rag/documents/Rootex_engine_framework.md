<!-- Source: https://rootex.readthedocs.io/en/latest/engine/framework.html -->

Framework — Rootex documentation
- Framework
# Framework
Rootex has certain funcitionalities built-in to support an ECS + Godot-like Scene implementation. The main aim of this kind of a architecture is to break down the game into a collection of behaviors and perceiving the game as an interplay of behaviors, rather than hardcoded functionalities inside each game object through object oriented programming. It originates from a place in the programming world that suggests “Composition is better than Inheritance”.
ECS just implements a dynamic composition, in the sense that behaviors can be added to game objects at runtime.
One of the popular side benefits of ECS is increased cache coherency.
Scenes allow users to organise and connect their entities into trees, providing powerful features to manipulate entity hierarchies, both at level design time and runtime.
Rootex’ ECS architecture has 3 main parts.
# Component
[Class Component](https://rootex.readthedocs.io/en/latest/api/class_component.html#class-component)
A Component is a collection of data for the game to use. Components are analogous to behaviors and components store some peculiar data to maintain their behavior. Component do not do anything else. They may allow changing the data in a certain manner from their public API.
Instances of a type of component are often iterated en masse. Hence all instances of a component type are stored in a special kind of array which allows max MAX_COMPONENT_ARRAY_SIZE instances to exist, but prevents changing location of any instance. The sole ownership of the components lies with this array. Everything else gets raw pointers to elements of this array.
Components can also register dependencies on other components, in form of hard or soft [Define DEPENDENCY](https://rootex.readthedocs.io/en/latest/api/define_component_8h_1a76d6d3f7803dbffb744669d8311c8e62.html#define-dependency). Hard dependency on a component means that the dependent component cannot function without it and creation of the component is blocked if the dependency isn’t fulfilled. Soft dependency is optional dependency, without which a component may be able to function, albeit limitedly and component creation is not blocked if a soft dependency is not fulfilled.
# Entity
[Class Entity](https://rootex.readthedocs.io/en/latest/api/class_entity.html#class-entity)
An Entity in Rootex is a collection of components. The entity will have a name additionally but all data being used in the game will be stored in one of the components of an entity. Entities provide the component with an identity so that components can be theorized to “belong” to a thing in the game. Entites can be globally identified from their IDs, which is guaranteed to be uniquely generated.
An Entity stores a HashMap of all the components assigned to it. The sole ownership(Ptr) of an entity belongs to it’s owner scene. Everything else gets raw pointers.
# System
[Class System](https://rootex.readthedocs.io/en/latest/api/class_system.html#class-system)
A System in Rootex is containing all the logic/algorithms that are needs to make sense of the data that is stored inside a specific type of component. Systems only interact with a certain type of components. In Rootex, all components of similar type are stored in an array and all these arrays containing different types of components are stored in a hash map so that the array having an component type can be indexed and used for processing by a _exhale_class_class_system.
Systems often require iterating over componets of a type, this can only be accomplished by using range based for loops as we have a custom iterator that only returns “valid” elements from our custom component array.
## Scene
A scene is a hierarchical data structure which can optionally store and entity. It stores children nodes of the same type and controls their lifetime based of if the parent is alive. Once the parent has decided to kill itself, it makes sure its children also meet the same fate.
Scene’s store a [Typedef Ptr](https://rootex.readthedocs.io/en/latest/api/typedef_types_8h_1a6b46abcd7303a8f484bd03805eb49bbc.html#typedef-ptr) to an entity and provide structure to out hybrid ECS + Scene architecture.
Rootex uses JSON style serializations to store and load scenes. Entities are also created from these files with all the necessary components.
A scene subtree can be saved to a file/loaded from a file to allow functionality reuse both during level design phase and runtime.
Scene files are recognized by .scene.json file extensions.
Scene construction is handled by [Class Scene](https://rootex.readthedocs.io/en/latest/api/class_scene.html#class-scene) itself but it delegates the entity construction using the JSON data to [Class ECSFactory](https://rootex.readthedocs.io/en/latest/api/class_e_c_s_factory.html#class-ecsfactory).
Each [Class Component](https://rootex.readthedocs.io/en/latest/api/class_component.html#class-component) accepts owner entity and constituent json data in its constructor, allowing it to setup its data memebrs from the serialised data. Every Component also defines a getJSON method which serialises its members back to JSON format. The data retention across engine restarts is guarranteed through ensuring that the component use the same data which it generates while saving the scene.
# Pausing
Pausing is tightly bound to ECS + Scenes. The engine provides a pause UI scene, which is enbled on pressing ESC key. All scenes which have the “Stop Scene during Pause” checkbox checked will have most of their components and scripts being skipped by Systems, effectively bringing the game logic to a pause. Certain scenes can be exempted from being skipped to allow stuff like Music to keep playing.
