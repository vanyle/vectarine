
local module = {}

local EventTypeImpl = {}
EventTypeImpl.__index = EventTypeImpl
export type Event<T> = typeof(setmetatable({}, EventTypeImpl)) & {__data_type: T} -- we put a fake T inside the event to make the system system work.

--- Create a new event type, for example `newEvent("hit")`
--- The name should be unique and is used for debugging purposes.
--- If you try to create newEvent with the same name, you will get the same event type, but
--- all its subscriptions will be cleared.
--- @param name string The name of the event type
--- @return EventType
function newEvent<T>(name: string): Event<T>
    error("Not implemented")
end
module.newEvent = newEvent  

--- Represents the type of all events
---
--- Subscribing to this event type will receive all events
local AnyEvent: Event<unknown>  = newEvent("any")
module.AnyEvent = AnyEvent 

--- Represents the type of no events
---
--- Subscribing to this event type never receives any events
local NeverEvent: Event<never>  = newEvent("any")
module.NeverEvent = NeverEvent 

--- Dispatch an event to the bus
--- @param event EventType<T> The type of event
--- @param data T Additional event data
function EventTypeImpl.dispatch<T>(event: Event<T>, data: T)
    error("Not implemented")
end

local SubIDImpl = {}
SubIDImpl.__index = SubIDImpl
--- @class SubID
--- A subscription ID that can be used to unsubscribe from an event
export type SubID = typeof(setmetatable({}, SubIDImpl))

--- Subscribe to an event type. When that event is triggered, the callback function will be called.
--- Note: When a file is reloaded, all subscriptions get auto-unsubscribed to prevent memory leaks.
--- @param event EventType<T> The type of event
--- @param callback fun(data: T) A function that will be called when the event is sent. The callback function will receive the event data as arguments.
--- @return SubID A subscription ID that can be used to unsubscribe
function EventTypeImpl.on<T>(event: Event<T>, callback: (data: T) -> ()): SubID
    error("Not implemented")
end

--- Clear all subscriptions for a given event type
--- @param event EventType<T> The type of event
function EventTypeImpl.clear<T>(event: Event<T>)
    error("Not implemented")
end

--- Stop receiving events for a given subscription ID
--- @param id SubID The subscription ID returned by `event:on(...)`
function SubIDImpl.unsubscribe(id: SubID)
    error("Not implemented")
end

return module