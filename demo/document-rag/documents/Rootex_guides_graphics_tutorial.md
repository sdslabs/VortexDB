<!-- Source: https://rootex.readthedocs.io/en/latest/guides/graphics_tutorial.html -->

Exploring the Graphical capabilities of Rootex — Rootex documentation
- Exploring the Graphical capabilities of Rootex
# Exploring the Graphical capabilities of Rootex
This documentation aims to showcase the graphical capabilities of rootex and act as a tutorial for beginners to get started.
Let’s start by creating a scene.
## Create a scene
To Create a Scene
1.
Go to file->CreateScene.
1.
Name the scene and click create.
Now we Create an Empty Scene. An empty scene is nothing but objects. You can have different components in it, more on that later.
## Create Empty scene
To create an empty scene.
1.
Right-click the root scene.
1.
Click Add Empty Scene
## Giving Components
Now we give components to the empty scene.
1.
Right-click the empty scene.
1.
Click Edit Components.
1.
Check the appropriate components, in this case, transform and Model. Note: Transform Component is a must.
1.
Open inspector.
1.
Go to the model component in the inspector.
1.
Click the folder icon next to Model.
1.
Select the sponza 3D model file located at `Rootex\game\assets\sponza\sponza.obj`
For sponza initially, it would look like this:
This is due to the default settings of the sponza obj file. To get a better view, set the scale to (0.031, 0.031, 0.031) and set the LOD distance to 123:
We need to create an empty scene and add a light component to it to add light.
## Light Component
To add light, we now create an empty scene.
1.
Name the scene.
1.
Add transform and directional light components.
To move freely, we can change our camera mode to Editor Camera. This allows us to move freely.
## Editor Camera
To have complete control of movement, you can use an editor camera.
1.
Click the figure icon at the top left of the viewport.
1.
Open dropdown of camera.
1.
Select editor camera.
To move, you have to hold the right mouse button and then use WASD space and shift keys to move. The cursor for direction. Space to move up and shift to move down.
## Point Light
A point light is helpful if you have a source of light, e.g. a candle, bulb etc. To add a point light, follow the given steps.
1.
Add an empty scene and give it a point light component.
1.
You can tweak its transformation value by either inputting it or dragging it left or right.
If you press ‘q’, a transform gizmo will appear on the object you have selected. You can adjust light location through it. For rotation and scaling gizmo, press ‘w’ and ‘e’, respectively.
## Overriding a material
To change the properties of one object without changing the original material, we can use overriding materials. To override a material:
1.
Create a new basic material by going to file -> Create Resource.
1.
Name the material and click create.
1.
Go to the `Inspector-> Model Component->Materials`.
1.
Click on the folder icon on the corresponding overriding material.
1.
Select the newly created basic material located at `Rootex\game\assets\materials\new_cloth.basic.rmat`
Now you can change its basic textures by 1)clicking on the pencil icon 2)In the file viewer now click on the diffuse texture and select the appropriate diffuse texture.
## Custom Material
1.
Go to create Resource -> Custom Material.
1.
Enter material name.
1.
Now go to Inspector -> ModelComponent and then to Materials.
1.
Click on the folder icon and choose the material.
## Adding a shader
To Add shader:
1.
Click on the pencil icon on the overriding custom material.
1.
Now, in the file viewer you’ll get options to add vertex and pixel shaders.
1.
Click on the pixel shader. A dialog box will open now you can just select the shader.
You can use fire_pixel_shader from rootex/core/renderer/shaders
Clicking on the pencil icon opens an editor to customise the shader.
Note
You can only add shaders to custom materials. If you want to use default material, override the original default material with custom material and then add a shader to the overriding material. The overriding material does inherit the textures of the original materials.
## Decal Component
To add a decal component.
1.
Make a scene DECAL and give it transform and Decal Component.
1.
Create a decal material. By going to File -> CreateResource. And then slect Decal material in resource type dropdown.
1.
Now go to the inspector and click DecalComponent.
1.
Click on the folder icon and select the decal material.
1.
Click on the pencil icon and the in the file viewer click on Decal Texture.
1.
Shift its position by manipulating the transform component.
By default, the decal shader projects on the negative z-axis. You can rotate it till you get the desired result.
