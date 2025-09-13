import { NextResponse } from 'next/server';

export async function GET() {
  const uptime = process.uptime();
  const version = process.env.npm_package_version || '1.0.0';
  
  // Prometheus format metrics
  const metrics = `# HELP tforce_frontend_uptime_seconds Total uptime in seconds
# TYPE tforce_frontend_uptime_seconds counter
tforce_frontend_uptime_seconds ${uptime}
# HELP tforce_frontend_version_info Version information
# TYPE tforce_frontend_version_info gauge
tforce_frontend_version_info{version="${version}"} 1
# HELP tforce_frontend_health_status Health status
# TYPE tforce_frontend_health_status gauge
tforce_frontend_health_status 1
`;

  return new NextResponse(metrics, {
    status: 200,
    headers: {
      'Content-Type': 'text/plain; version=0.0.4; charset=utf-8',
    },
  });
} 