local api = vim.api
local fn = vim.fn
local loop = vim.loop

vim.opt.termguicolors = true
vim.opt.eventignore = 'all'

local injected_rtp = vim.env.MEOW_RTP
if injected_rtp and injected_rtp ~= "" then
    local paths = vim.split(injected_rtp, ',')
    for _, p in ipairs(paths) do vim.opt.rtp:append(p) end
else
    local packpaths = vim.split(vim.o.packpath, ',')
    for _, pp in ipairs(packpaths) do
        local real_pp = loop.fs_realpath(pp) or pp
        for _, p in ipairs(fn.glob(real_pp .. '/pack/*/start/*', true, true)) do vim.opt.rtp:append(p) end
        for _, p in ipairs(fn.glob(real_pp .. '/pack/*/opt/*', true, true)) do vim.opt.rtp:append(p) end
    end
end

local theme = vim.env.MEOW_THEME or "habamax"
if not pcall(vim.cmd, 'colorscheme ' .. theme) then
    vim.cmd('colorscheme habamax')
end

if vim.fn.exists("g:syntax_on") == 0 then vim.cmd('syntax on') end

vim.cmd('silent! filetype detect')

local ft = vim.bo.filetype
if ft and ft ~= "" then
    vim.b.current_syntax = nil
    pcall(vim.cmd, 'runtime! syntax/' .. ft .. '.vim')
end

local ts_active = false
local ts_ok, ts = pcall(require, 'vim.treesitter')
if ts_ok and pcall(ts.start) then ts_active = true end
vim.cmd('redraw!')

local hl_cache = {}
local line_cache = {}
local marks_by_line = {}

local function get_ansi(hl_id)
    if not hl_id or hl_id <= 0 then return "" end
    if hl_cache[hl_id] then return hl_cache[hl_id] end
    
    local hl = api.nvim_get_hl(0, { id = hl_id, link = false })
    if (not hl.fg and not hl.bg) then hl = api.nvim_get_hl(0, { id = hl_id, link = true }) end

    local parts = {}
    if hl.bold then table.insert(parts, "1") end
    if hl.italic then table.insert(parts, "3") end
    
    local function rgb(c)
        return bit.rshift(c, 16), bit.band(bit.rshift(c, 8), 0xFF), bit.band(c, 0xFF)
    end
    if hl.fg then
        local r, g, b = rgb(hl.fg)
        table.insert(parts, string.format("38;2;%d;%d;%d", r, g, b))
    end
    if hl.bg then
        local r, g, b = rgb(hl.bg)
        table.insert(parts, string.format("48;2;%d;%d;%d", r, g, b))
    end
    
    local code = (#parts == 0) and "" or ("\27[" .. table.concat(parts, ";") .. "m")
    hl_cache[hl_id] = code
    return code
end

if ts_active then
    local all_marks = api.nvim_buf_get_extmarks(0, -1, 0, -1, {details=true})
    for _, m in ipairs(all_marks) do
        local row = m[2] + 1
        local col = m[3] + 1
        local det = m[4]
        if det.hl_group then
            if not marks_by_line[row] then marks_by_line[row] = {} end
            local end_col = (det.end_col and det.end_col + 1) or (col + 1)
            local hl_id = (type(det.hl_group) == "string") and api.nvim_get_hl_id_by_name(det.hl_group) or det.hl_group
            table.insert(marks_by_line[row], {col, end_col, hl_id})
        end
    end
end

local lines = api.nvim_buf_get_lines(0, 0, -1, false)
local output = {}

for i, line in ipairs(lines) do
    if #line == 0 then
        table.insert(output, "")
    else
        if line_cache[line] then
            table.insert(output, line_cache[line])
        else
            local buffer = {}
            local ts_marks = marks_by_line[i]
            
            if ts_marks then
                table.sort(ts_marks, function(a, b) return a[1] < b[1] end)
                local current_col = 1
                for _, m in ipairs(ts_marks) do
                    local start_c, end_c, id = m[1], m[2], m[3]
                    if start_c >= current_col then
                        if start_c > current_col then table.insert(buffer, string.sub(line, current_col, start_c - 1)) end
                        table.insert(buffer, get_ansi(id))
                        table.insert(buffer, string.sub(line, start_c, end_c - 1))
                        table.insert(buffer, "\27[0m")
                        current_col = end_c
                    end
                end
                if current_col <= #line then table.insert(buffer, string.sub(line, current_col)) end
            else
                local last_id = -1
                local chunk_start = 1
                for col = 1, #line do
                    local id = fn.synID(i, col, 1)
                    if id ~= last_id then
                        if col > chunk_start then
                            if last_id > 0 then table.insert(buffer, get_ansi(last_id)) end
                            table.insert(buffer, string.sub(line, chunk_start, col - 1))
                        end
                        chunk_start = col
                        last_id = id
                    end
                end
                if last_id > 0 then table.insert(buffer, get_ansi(last_id)) end
                table.insert(buffer, string.sub(line, chunk_start))
                table.insert(buffer, "\27[0m")
            end
            
            local res = table.concat(buffer)
            line_cache[line] = res
            table.insert(output, res)
        end
    end
end

io.stdout:write(table.concat(output, "\n"))
io.stdout:write("\n")
vim.cmd('qa!')
