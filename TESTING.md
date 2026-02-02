# Canopy Manual Testing Plan

## Milestone 1 Testing Checklist

### CLI Testing
- [ ] `canopy --help` shows correct usage
- [ ] `canopy index` successfully indexes a test repository
- [ ] `canopy index --verbose` shows detailed logging
- [ ] `canopy clear` properly clears cache
- [ ] `canopy version` displays version info

### Server Testing
- [ ] Server starts on default port 7890
- [ ] WebSocket connection established successfully
- [ ] HTTP API returns graph data
- [ ] Real-time updates via WebSocket work

### Browser Client Testing
- [ ] HTML page loads correctly in Chrome
- [ ] Graph visualization renders nodes and edges
- [ ] Nodes are clickable and show details
- [ ] Zoom/pan controls work
- [ ] Directory expand/collapse works
- [ ] Change indicators appear on file modifications

### File Watching Testing
- [ ] File watcher detects new files
- [ ] File modifications trigger updates
- [ ] Deleted files are removed from graph
- [ ] Changes appear in browser within 200ms

### Language Extraction Testing
- [ ] Rust files are parsed correctly
- [ ] TypeScript files are parsed correctly
- [ ] Functions/classes are identified
- [ ] Import relationships are detected

### Browser Compatibility
- [ ] Test in Chrome (primary)
- [ ] Test in Firefox
- [ ] Test in Safari
- [ ] Test responsive design

### Performance Testing
- [ ] Index 1000+ files without issues
- [ ] Graph renders with 100+ nodes smoothly
- [ ] WebSocket updates don't lag
- [ ] Memory usage stays reasonable

## Test Commands

```bash
# Start server
cargo run -- serve

# Test indexing
cargo run -- --verbose index

# Test with different repos
cd /path/to/test/repo
cargo run -- index

# Test file watching
cargo run -- serve &
echo "// test" >> src/main.rs  # Should trigger update
```

## Manual Browser Testing Steps

1. **Basic Load Test**
   - Navigate to http://localhost:7890
   - Verify page loads without errors
   - Check browser console for any warnings

2. **Graph Visualization Test**
   - Verify nodes are visible
   - Check edges connect nodes properly
   - Test node colors (directories vs files)
   - Verify labels are readable

3. **Interaction Test**
   - Click on a directory node
   - Click on a file node
   - Test sidebar shows correct details
   - Verify zoom controls work

4. **Real-time Updates Test**
   - Open browser to localhost:7890
   - In terminal, modify a file
   - Verify change indicator appears
   - Check update appears within 200ms

5. **Stress Test**
   - Open multiple browser tabs
   - Make rapid file changes
   - Verify no memory leaks
   - Check performance remains smooth

## Browser Dev Tools Testing

1. **Network Tab**
   - Verify WebSocket connection established
   - Check no 404 errors for assets
   - Monitor message frequency

2. **Console Tab**
   - Check for JavaScript errors
   - Verify WebSocket messages logged
   - Look for any warnings

3. **Performance Tab**
   - Record performance during updates
   - Check for long-running scripts
   - Monitor memory usage

4. **Elements Tab**
   - Inspect SVG structure
   - Verify proper DOM updates
   - Check CSS classes applied correctly
