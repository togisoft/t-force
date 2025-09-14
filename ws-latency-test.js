#!/usr/bin/env node

const WebSocket = require('ws');
const https = require('https');
const http = require('http');

class WebSocketLatencyTester {
    constructor() {
        this.authToken = null;
        this.ws = null;
        this.latencies = [];
    }

    async login(email = 'admin@example.com', password = 'Admin123!') {
        return new Promise((resolve, reject) => {
            const postData = JSON.stringify({ email, password });
            
            const options = {
                hostname: 'localhost',
                port: 80,
                path: '/api/auth/login',
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Content-Length': Buffer.byteLength(postData)
                }
            };

            const req = http.request(options, (res) => {
                let data = '';
                
                res.on('data', (chunk) => {
                    data += chunk;
                });
                
                res.on('end', () => {
                    if (res.statusCode === 200) {
                        try {
                            const response = JSON.parse(data);
                            this.authToken = response.token;
                            console.log('✅ Login successful!');
                            resolve(response);
                        } catch (e) {
                            reject(new Error('Invalid JSON response'));
                        }
                    } else {
                        reject(new Error(`Login failed: ${res.statusCode} - ${data}`));
                    }
                });
            });

            req.on('error', (error) => {
                reject(error);
            });

            req.write(postData);
            req.end();
        });
    }

    connectWebSocket() {
        return new Promise((resolve, reject) => {
            if (!this.authToken) {
                reject(new Error('No authentication token available'));
                return;
            }

            const wsUrl = `ws://localhost/api/chat/ws?token=${encodeURIComponent(this.authToken)}`;
            console.log(`Connecting to WebSocket: ${wsUrl}`);
            
            this.ws = new WebSocket(wsUrl);
            
            this.ws.on('open', () => {
                console.log('✅ WebSocket connection successful');
                resolve();
            });
            
            this.ws.on('error', (error) => {
                console.error('❌ WebSocket error:', error);
                reject(error);
            });
            
            this.ws.on('close', () => {
                console.log('🔌 WebSocket connection closed');
            });
            
            this.ws.on('message', (data) => {
                this.handleMessage(data);
            });
        });
    }

    handleMessage(data) {
        try {
            const message = JSON.parse(data.toString());
            
            if (message.message_type === 'pong' && message.data && message.data.timestamp) {
                const endTime = Date.now();
                const latency = endTime - message.data.timestamp;
                
                this.latencies.push(latency);
                console.log(`Ping ${this.latencies.length}: ${latency.toFixed(2)}ms`);
            }
        } catch (e) {
            // Not a JSON message or not a pong message
        }
    }

    async pingPongTest(count = 10) {
        console.log(`\n🚀 Starting Ping-Pong test (${count} pings)...`);
        
        for (let i = 0; i < count; i++) {
            const startTime = Date.now();
            const pingId = `ping_${startTime}_${Math.random()}`;
            
            this.ws.send(JSON.stringify({
                type: 'ping',
                id: pingId,
                timestamp: startTime
            }));
            
            // Wait for pong
            await new Promise(resolve => setTimeout(resolve, 1000));
        }
        
        this.showStatistics();
    }

    async continuousTest(duration = 30) {
        console.log(`\n🚀 Starting continuous test for ${duration} seconds...`);
        
        const startTime = Date.now();
        const endTime = startTime + (duration * 1000);
        
        while (Date.now() < endTime) {
            const pingStartTime = Date.now();
            const pingId = `continuous_${pingStartTime}_${Math.random()}`;
            
            this.ws.send(JSON.stringify({
                type: 'ping',
                id: pingId,
                timestamp: pingStartTime
            }));
            
            await new Promise(resolve => setTimeout(resolve, 2000));
        }
        
        this.showStatistics();
    }

    async batchTest(count = 10) {
        console.log(`\n🚀 Starting batch test (${count} pings)...`);
        
        for (let i = 0; i < count; i++) {
            const startTime = Date.now();
            const pingId = `batch_${i}_${startTime}`;
            
            this.ws.send(JSON.stringify({
                type: 'ping',
                id: pingId,
                timestamp: startTime
            }));
            
            // Short delay between pings
            await new Promise(resolve => setTimeout(resolve, 100));
        }
        
        // Wait a bit for all responses
        await new Promise(resolve => setTimeout(resolve, 1000));
        
        this.showStatistics();
    }

    showStatistics() {
        if (this.latencies.length === 0) {
            console.log('❌ No latency data collected');
            return;
        }
        
        const avg = this.latencies.reduce((a, b) => a + b, 0) / this.latencies.length;
        const min = Math.min(...this.latencies);
        const max = Math.max(...this.latencies);
        
        console.log('\n📊 Statistics:');
        console.log(`Average: ${avg.toFixed(2)}ms`);
        console.log(`Minimum: ${min.toFixed(2)}ms`);
        console.log(`Maximum: ${max.toFixed(2)}ms`);
        console.log(`Total tests: ${this.latencies.length}`);
        
        // Performance rating
        if (avg < 50) {
            console.log('🟢 Performance: Excellent');
        } else if (avg < 100) {
            console.log('🟡 Performance: Good');
        } else if (avg < 200) {
            console.log('🟠 Performance: Fair');
        } else {
            console.log('🔴 Performance: Poor');
        }
    }

    close() {
        if (this.ws) {
            this.ws.close();
        }
    }
}

// Main execution
async function main() {
    const tester = new WebSocketLatencyTester();
    
    try {
        // Login
        await tester.login();
        
        // Connect WebSocket
        await tester.connectWebSocket();
        
        // Run tests based on command line arguments
        const args = process.argv.slice(2);
        const testType = args[0] || 'pingpong';
        const count = parseInt(args[1]) || 10;
        
        switch (testType.toLowerCase()) {
            case 'pingpong':
                await tester.pingPongTest(count);
                break;
            case 'continuous':
                await tester.continuousTest(count);
                break;
            case 'batch':
                await tester.batchTest(count);
                break;
            default:
                console.log('Usage: node ws-latency-test-auth-en.js [pingpong|continuous|batch] [count]');
                console.log('Examples:');
                console.log('  node ws-latency-test-auth-en.js pingpong 20');
                console.log('  node ws-latency-test-auth-en.js continuous 60');
                console.log('  node ws-latency-test-auth-en.js batch 15');
        }
        
    } catch (error) {
        console.error('❌ Error:', error.message);
    } finally {
        tester.close();
    }
}

// Run if this file is executed directly
if (require.main === module) {
    main();
}

module.exports = WebSocketLatencyTester;
