return function(ctx)
 if ctx.event == "action.pace" then
  local run = not ctx.state["harbor.running"]
  return {{op="set",key="harbor.running",value=run},{op="pace",entity="$player",speed=run and 112 or 68}}
 end
 if ctx.event == "tick" and ctx.scene == "harbor" and not ctx.state["harbor.crate"] and entity("crate").position[1] >= 208 then
  return {{op="set",key="harbor.crate",value=true},{op="objective",quest="workshop",objective="crate"}}
 end
 return {}
end
