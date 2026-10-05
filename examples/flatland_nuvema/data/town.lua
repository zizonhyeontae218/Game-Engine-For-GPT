return function(ctx)
  if ctx.event == "action.pace" then
    local run = not ctx.state["town.running"]
    return {{op="set",key="town.running",value=run},{op="pace",entity="$player",speed=run and 72 or 48}}
  end
  if ctx.event == "tick" and ctx.scene == "lab" and not ctx.state["town.crate"] then
    local x = entity("crate").position[1]
    if x >= 80 then return {{op="set",key="town.crate",value=true},{op="objective",quest="research",objective="crate"}} end
  end
  return {}
end
