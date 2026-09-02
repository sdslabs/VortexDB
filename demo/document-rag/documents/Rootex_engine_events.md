<!-- Source: https://rootex.readthedocs.io/en/latest/engine/events.html -->

Event Manager — Rootex documentation
- Event Manager
# Event Manager
Events are a way to solve the problem of sphagetti code, and an efficient way to implement the publisher-subscriber design pattern in a considerably large codebase.
Events ([Class Event](https://rootex.readthedocs.io/en/latest/api/class_event.html#class-event)) in Rootex are the equivalent of broadcast messages of a particular channel and Rootex’ Event Manager ([Class EventManager](https://rootex.readthedocs.io/en/latest/api/class_event_manager.html#class-eventmanager)) is the equivalent of a broadcast company managing multiple channels. Functions can be registered as callbacks to events. When an event is called, all the functions registered to that event are called with the corresponding data related to the cause of origin of that event. Rootex’ event manager has the ability to call both global functions and member functions (with the corresponding object that registered its member function, as a parameter into the member function, which is how C++ implements member functions).
[Class EventManager](https://rootex.readthedocs.io/en/latest/api/class_event_manager.html#class-eventmanager) is a singleton, and all engine level events are passed by it. User events can also be channeled through with no issues. E.g. Input events that are configured by the user are sent through the engine level event manager.
Optionally EventManager allows [Typedef Variant](https://rootex.readthedocs.io/en/latest/api/typedef_types_8h_1ab10036d197bc23eea4d105ef1d9026b1.html#typedef-variant) data to be passed along with an event call, which are further of 2 types:
-
Call: The registered handlers are called immediately. This is useful for non destructive activity.
-
Deferred Call: The registered handlers are called at the end of a frame. This is especially useful for destrutive activity related to entities, components and systems, as the engine is very likely to be iterating on them and deletion may lead to corruption.
Rootex editor has been made on top of the rootex engine. The engine is never aware of the existence of editor. This is made possible by the help of events.
See [Struct RootexEvents](https://rootex.readthedocs.io/en/latest/api/struct_rootex_events.html#struct-rootexevents), [Struct EditorEvents](https://rootex.readthedocs.io/en/latest/api/struct_editor_events.html#struct-editorevents), [Class Event](https://rootex.readthedocs.io/en/latest/api/class_event.html#class-event)
