// @ts-check
import react from "@astrojs/react";
import starlight from "@astrojs/starlight";
import expressiveCode from "astro-expressive-code";
import { defineConfig } from "astro/config";
import starlightBlog from 'starlight-blog';
import starlightKbd from "starlight-kbd";

// https://astro.build/config
export default defineConfig({
    srcDir: "./src",
    // Cloudflare prefers trailing slashes it seems, but setting it to always breaks RSS, so we set it to ignore.
    trailingSlash: "ignore",
    build: {
        format: "directory",
    },
    base: "/",
    site: "https://vectarineengine.com",
    integrations: [
        expressiveCode(),
        starlight({
            title: "Vectarine",
            titleDelimiter: " | ",
            tagline: "The cross-platform Luau game engine focusing on ultra fast prototyping and having fun.",
            favicon: "vectarine.png",
            customCss: ["./src/styles/custom.css"],
            logo: {
                src: "./src/assets/vectarine.png",
                alt: "Vectarine Logo",
            },
            components: {
                Header: "./src/components/Header.astro",
                SocialIcons: "./src/components/SocialIcons.astro",
                SiteTitle: "./src/components/SiteTitle.astro",
                Hero: "./src/components/Hero.astro",
                PageFrame: "./src/components/PageFrame.astro",
            },
            editLink: {
                baseUrl: 'https://github.com/vanyle/vectarine/edit/main/docs/website',
            },
            plugins: [
                starlightBlog(),
                starlightKbd({
                    types: [
                        { id: "mac", label: "macOS", default: true },
                        { id: "windows", label: "Windows" },
                    ],
                }),
            ],
            social: [
                {
                    icon: "github",
                    label: "GitHub",
                    href: "https://github.com/vanyle/vectarine",
                },
                {
                    icon: "discord",
                    label: "Discord",
                    href: "https://discord.gg/zPwg3VDydz",
                },
            ],
            sidebar: [
                {
                    label: "Gallery",
                    items: [
                        {
                            label: "What is the Gallery?",
                            link: "/gallery/",
                        },
                        {
                            label: "Snake",
                            link: "/gallery/snake/",
                        },
                        {
                            label: "Sokoban",
                            link: "/gallery/sokoban/",
                        },
                        {
                            label: "Xmas AAA",
                            link: "/gallery/xmas-3a/",
                        },
                    ],
                },
                {
                    label: "Introductions",
                    items: [
                        {
                            label: "Create your first game",
                            link: "/guides/getting-started/",
                        },
                        {
                            label: "Overview",
                            link: "/guides/overview/",
                        },
                    ],
                },
                {
                    label: "Guides",
                    items: [
                        {
                            label: "Drawing images and levels with tilesets",
                            link: "/guides/tilemaps-and-tilesets/",
                        },
                        {
                            label: "Understanding hot-reloading",
                            link: "/guides/understanding-hotreloading/",
                        },
                        {
                            label: "Making a platformer",
                            link: "/guides/making-a-platformer/",
                        },
                        {
                            label: "Optimizing your game",
                            link: "/guides/use-fastlists/",
                        },
                        {
                            label: "Making user interfaces",
                            link: "/guides/making-uis/",
                        },
                        {
                            label: "Testing games automatically",
                            link: "/guides/testing-games-automatically/",
                        },
                        {
                            label: "Create a native plugin",
                            link: "/guides/create-a-plugin/",
                        },
                        {
                            label: "Design Principles",
                            link: "/guides/design-principles/",
                        },
                        {
                            label: "FAQ",
                            link: "/guides/faq/",
                        }
                    ],
                },
            ],
        }),
        react(),
    ],
});
