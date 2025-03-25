# Dynamit (Dynamic + Explosive = Explosiv Dynamic System)

Currently this is in a pre-pre-pre-alfa version!

Parsing a *HTML-like* Content into a *VDOM*, run *Lua Scripts* to *manipulate Elements*, implement a render-backend / create a *RenderBackend Trait* to convert VDOM to a RenderCommandList like thing

- Make `<template>` elements to make it reusable by scripting

- run *lua* script in the `onload` event of the body to call a function at initialization

- Use custom lua function to make webrequest and parse webrequest, create_elements from templates and add it to a specific parent:

```lua

function load_data()
    -- 🌍 Daten abrufen (simulierte API-Anfrage)
    local json = '[{"city": "Berlin", "temp": 21}, {"city": "Hamburg", "temp": 18}, {"city": "München", "temp": 24}]'
    local weather_data = parse_json(json)

    -- 🌱 Für jedes Wetter-Daten-Objekt eine Karte erstellen
    for _, entry in ipairs(weather_data) do
        local card = create_element("weather_card", "weather_list") -- 🎨 Template klonen
        card:set_text("city", entry.city)
        card:set_text("temp", tostring(entry.temp))
    end
end

```

## what next

- create onupdate function witch ticked regualary
- create simple opengl example to render content on screen
- use nanovg for simple shader drawing
- implement dirty-tracker (only rerender content that really changed)
