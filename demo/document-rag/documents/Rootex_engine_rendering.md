<!-- Source: https://rootex.readthedocs.io/en/latest/engine/rendering.html -->

Rendering — Rootex documentation
- Rendering
# Rendering
Rootex uses DirectX 11 to render graphics.
Rendering in Rootex has been implemented with special attention so that it behaves properly with our ECS architecture. The [Class RenderSystem](https://rootex.readthedocs.io/en/latest/api/class_render_system.html#class-rendersystem) uses the [Class RenderableComponent](https://rootex.readthedocs.io/en/latest/api/class_renderable_component.html#class-renderablecomponent) to share common funcitonalities across different components which add to the visuals of the scene.
The [Class RenderSystem](https://rootex.readthedocs.io/en/latest/api/class_render_system.html#class-rendersystem) uses the owning [Class Scene](https://rootex.readthedocs.io/en/latest/api/class_scene.html#class-scene) of the renderable component to recursively traverse the object hierarchy, starting from the root entity (which is persistent across levels). Every time the render system recognizes a parent, before processing its children, the render system takes note of the transform (a representation of position, rotation and scale all at once) of the parent and appends it to the transformation stack. The transformation stack is an implementation for inheriting transforms from the parent entity of a child entity, used while performing a Depth-First-Search on the component hierarchy established by hierarchy component instances.
The transformation stack of UI components is kept separate from the transformation stack of 3D world visual components.
Once all transformations are updated, [Class RenderSystem](https://rootex.readthedocs.io/en/latest/api/class_render_system.html#class-rendersystem) loops over all the renderable components and does the rendering required to show them.
Rootex also performs sky, fog and related rendering effects and post processing effects.
