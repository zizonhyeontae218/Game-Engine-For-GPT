local signals = require("signals")
return function(ctx)
  if ctx.event == "start" then return {{op="set",key="yard.roll",value=signals.roll()}} end
  return {}
end
