pub const LUA_GENERATOR: &str = r#"
local fn = vim.fn
local api = vim.api
local file = vim.env.MEOW_FILE

-- 1. Open file silently
vim.cmd('silent! edit ' .. file)

-- 2. Cache Setup
local hl_cache = {}    -- ID -> ANSI Code
local trans_cache = {} -- Raw ID -> Translated ID (The new optimization)

local function rgb_to_ansi(r, g, b)
    return string.format("38;2;%d;%d;%d", r, g, b)
end
local function bg_to_ansi(r, g, b)
    return string.format("48;2;%d;%d;%d", r, g, b)
end

local function get_ansi(hl_id)
    if not hl_id or hl_id <= 0 then return "" end
    if hl_cache[hl_id] then return hl_cache[hl_id] end

    local ok, hl = pcall(api.nvim_get_hl, 0, { id = hl_id, link = false })
    if not ok or not hl then
        hl_cache[hl_id] = ""
        return ""
    end

    local parts = {}
    if hl.bold then table.insert(parts, "1") end
    if hl.italic then table.insert(parts, "3") end
    if hl.underline then table.insert(parts, "4") end

    if hl.fg then
        local r = bit.rshift(hl.fg, 16)
        local g = bit.band(bit.rshift(hl.fg, 8), 0xFF)
        local b = bit.band(hl.fg, 0xFF)
        table.insert(parts, rgb_to_ansi(r, g, b))
    end

    if hl.bg then
        local r = bit.rshift(hl.bg, 16)
        local g = bit.band(bit.rshift(hl.bg, 8), 0xFF)
        local b = bit.band(hl.bg, 0xFF)
        table.insert(parts, bg_to_ansi(r, g, b))
    end

    local code = (#parts == 0) and "" or ("\27[" .. table.concat(parts, ";") .. "m")
    hl_cache[hl_id] = code
    return code
end

local lines = api.nvim_buf_get_lines(0, 0, -1, false)
local output = {}

for i, line in ipairs(lines) do
    if #line == 0 then
        table.insert(output, "")
    else
        -- PERFORMANCE GUARD: Skip highlighting for minified/long lines
        if #line > 1000 then
            table.insert(output, line)
        else
            local line_buffer = {}
            local last_id = -1
            local chunk_start = 1
            local line_len = #line
            local line_str = line

            for col = 1, line_len do
                -- OPTIMIZATION: 2 C-calls -> 1 C-call + Table Lookup
                local raw_id = fn.synID(i, col, 1)
                
                -- Check cache first
                local id = trans_cache[raw_id]
                if not id then
                    -- Cache miss: Call C function and store
                    id = fn.synIDtrans(raw_id)
                    trans_cache[raw_id] = id
                end
                
                if id ~= last_id then
                    if col > chunk_start then
                        if last_id > 0 then
                            table.insert(line_buffer, get_ansi(last_id))
                        end
                        table.insert(line_buffer, string.sub(line_str, chunk_start, col - 1))
                    end
                    chunk_start = col
                    last_id = id
                end
            end

            if last_id > 0 then
                 table.insert(line_buffer, get_ansi(last_id))
            end
            table.insert(line_buffer, string.sub(line_str, chunk_start, line_len))
            table.insert(line_buffer, "\27[0m")
            
            table.insert(output, table.concat(line_buffer))
        end
    end
end

io.stdout:write(table.concat(output, "\n"))
io.stdout:write("\n")
vim.cmd('qa!')
"#;
