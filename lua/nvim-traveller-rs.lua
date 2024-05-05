--local uv = vim.uv

--local function setTimeout(timeout, callback)
  --local timer = uv.new_timer()
  --timer:start(timeout, 0, function ()
    --timer:stop()
    --timer:close()
    --callback()
  --end)
  --return timer
--end

--setTimeout(3000, function ()
  --vim.print("jup")
--end)

return require("nvim_traveller_rs")
