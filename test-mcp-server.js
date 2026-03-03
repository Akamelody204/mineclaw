#!/usr/bin/env node

/**
 * 简单的测试用 MCP 服务器
 * 提供一个 echo 工具
 */

console.error = (...args) => {
    process.stderr.write(args.join(' ') + '\n');
};

console.error('[test-server] Starting...');

let buffer = '';

process.stdin.on('data', (data) => {
    buffer += data.toString();
    
    // 按行处理
    const lines = buffer.split('\n');
    buffer = lines.pop() || '';
    
    for (const line of lines) {
        if (line.trim()) {
            handleMessage(line);
        }
    }
});

function handleMessage(line) {
    console.error('[test-server] Received:', line);
    
    try {
        const request = JSON.parse(line);
        
        if (request.method === 'initialize') {
            console.error('[test-server] Handling initialize');
            const response = {
                jsonrpc: '2.0',
                id: request.id,
                result: {
                    protocolVersion: '2024-11-05',
                    capabilities: {
                        tools: {}
                    },
                    serverInfo: {
                        name: 'test-server',
                        version: '1.0.0'
                    }
                }
            };
            sendResponse(response);
        } 
        else if (request.method === 'tools/list') {
            console.error('[test-server] Handling tools/list');
            const response = {
                jsonrpc: '2.0',
                id: request.id,
                result: {
                    tools: [
                        {
                            name: 'echo',
                            description: 'Echo back the input message',
                            inputSchema: {
                                type: 'object',
                                properties: {
                                    message: { type: 'string' }
                                },
                                required: ['message']
                            }
                        },
                        {
                            name: 'add',
                            description: 'Add two numbers',
                            inputSchema: {
                                type: 'object',
                                properties: {
                                    a: { type: 'number' },
                                    b: { type: 'number' }
                                },
                                required: ['a', 'b']
                            }
                        }
                    ]
                }
            };
            sendResponse(response);
        }
        else if (request.method === 'initialized') {
            console.error('[test-server] Received initialized notification');
            // 通知不需要响应
        }
        else if (request.method === 'tools/call') {
            console.error('[test-server] Handling tools/call');
            const toolName = request.params?.name;
            const args = request.params?.arguments || {};
            
            let content = [];
            let isError = false;
            
            if (toolName === 'echo') {
                const message = args?.message;
                if (message) {
                    content = [{
                        type: 'text',
                        text: message
                    }];
                } else {
                    content = [{
                        type: 'text',
                        text: 'Error: message parameter is required'
                    }];
                    isError = true;
                }
            } else if (toolName === 'add') {
                const a = args?.a;
                const b = args?.b;
                if (typeof a === 'number' && typeof b === 'number') {
                    content = [{
                        type: 'text',
                        text: String(a + b)
                    }];
                } else {
                    content = [{
                        type: 'text',
                        text: 'Error: a and b must be numbers'
                    }];
                    isError = true;
                }
            } else {
                content = [{
                    type: 'text',
                    text: `Error: Unknown tool ${toolName}`
                }];
                isError = true;
            }
            
            const response = {
                jsonrpc: '2.0',
                id: request.id,
                result: {
                    content: content,
                    isError: isError
                }
            };
            sendResponse(response);
        }
        else {
            console.error('[test-server] Unknown method:', request.method);
            const response = {
                jsonrpc: '2.0',
                id: request.id,
                error: {
                    code: -32601,
                    message: 'Method not found'
                }
            };
            sendResponse(response);
        }
    } catch (e) {
        console.error('[test-server] Error:', e);
    }
}

function sendResponse(response) {
    const json = JSON.stringify(response);
    console.error('[test-server] Sending:', json);
    process.stdout.write(json + '\n');
}

process.stdin.on('end', () => {
    console.error('[test-server] stdin closed, exiting');
    process.exit(0);
});