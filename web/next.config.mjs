/** @type {import('next').NextConfig} */
const nextConfig = {
    images: {
        // All images come from the auth-gated API. Next's image optimizer runs server-side
        // and can't carry the user's session, so it gets redirected to login and reports
        // "The requested resource isn't a valid image". Serve images unoptimized so the
        // authenticated browser fetches them directly. (Many workshop illustrations are SVG,
        // which the optimizer wouldn't resize anyway.)
        unoptimized: true,
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
    output: 'standalone',
};

export default nextConfig;
