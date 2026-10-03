-- Lua source runs in the restricted engine VM. State belongs to Rust, not globals.
return function(ctx)
  if ctx.event == "pickup" then
    return {{op="add", key="game.pickups", value=1}}
  end
  return {}
end
