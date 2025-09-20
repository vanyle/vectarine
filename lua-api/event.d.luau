
--- @class EventType<T>: {__data_type: T}
--- @field private __data_type `T` The type of data associated with the event
--- Represents a type of event
--- @operator add(EventType): EventType -- Combine two event types into a new one
--- A type of event
local Event = {}

--- Represents the type of all events
---
--- Subscribing to this event type will receive all events
--- @type EventType<nil>
AnyEvent = { __data_type = "never" }

--- Represents the type of no events
---
--- Subscribing to this event type will never receive any events
--- Bus.send(Bus.NeverEvent) is a no-op
--- @type EventType<any>
NeverEvent = { __data_type = "any" }

--- Create a new event type, for example `newEvent("number")`
--- @generic T
--- @param dataType `T` The type of data associated with the event type
--- @return EventType T
function newEvent(dataType) end

--- Dispatch an event to the bus
--- @generic T
--- @param event EventType<T> The type of event
--- @param data T Additional event data
function Event.dispatch(event, data) end

--- @class SubID
--- A subscription ID that can be used to unsubscribe from an event
local SubID = {}

--- Subscribe to an event type. When that event is triggered, the callback function will be called.
--- Note: When a file is reloaded, all subscriptions get auto-unsubscribed to prevent memory leaks.
--- @generic T
--- @param event EventType<T> The type of event
--- @param callback fun(data: T) A function that will be called when the event is sent. The
--- callback function will receive the event data as arguments.
--- @return SubID A subscription ID that can be used to unsubscribe
function Event.on(event, callback) end

--- Unsubscribe from an event type
--- @param event EventType The type of event
--- @param id SubID The subscription ID returned by `Bus.when`
function Event.unsubscribe(event, id) end
