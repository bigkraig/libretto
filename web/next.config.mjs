/** @type {import('next').NextConfig} */
const nextConfig = {
    webpack: {
        config: {
            resolve: {
                alias: {
                    canvas: false,
                }
            }
        }
    },
    images: {
        dangerouslyAllowSVG: true,
        remotePatterns: [
            {
                protocol: 'https',
                hostname: 'libretto.bigkraig.com',
                port: '',
                pathname: '/**',
            },
            {
                protocol: 'http',
                hostname: 'libretto-api',
                port: '3030',
                pathname: '/v1/**',
            },
            {
                protocol: 'http',
                hostname: 'localhost',
                port: '3030',
                pathname: '/**',
            },
        ],
    },
    experimental: {
        missingSuspenseWithCSRBailout: false,
    },
    output: 'standalone',
};

export default nextConfig;
