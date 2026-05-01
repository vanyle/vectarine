// @ts-check
import react from "@astrojs/react";
import starlight from "@astrojs/starlight";
import expressiveCode from "astro-expressive-code";
import { defineConfig } from "astro/config";

// https://astro.build/config
export default defineConfig({
    srcDir: "./src",
    build: {
        format: "file",
    },
    base: "/",
    site: 'https://vectarine.surge.sh',
    integrations: [
        expressiveCode(),
        starlight({
            title: "Vectarine",
            favicon: "vectarine.png",
            customCss: ["./src/styles/custom.css"],
            components: {
                SocialIcons: "./src/components/SocialIcons.astro",
            },
            social: [
                {
                    icon: "github",
                    label: "GitHub",
                    href: "https://github.com/vanyle/vectarine",
                },
                {
                    icon: 'discord',
                    label: 'Discord',
                    href: 'https://discord.gg/zPwg3VDydz'
                },
            ],
            sidebar: [
                {
                    label: "Welcome",
                    link: "/",
                },
                {
                    label: "Introductions",
                    items: [
                        {
                            label: "Create your first game",
                            link: "/guides/getting-started/",
                        },
                        {
                            label: "A guided tour",
                            link: "/guides/a-guided-tour/",
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
                            label: "Optimizing your game",
                            link: "/guides/use-fastlists/",
                        },
                        {
                            label: "Making user interfaces",
                            link: "/guides/making-uis/",
                        },
                                                {
                            label: "Create a native plugin",
                            link: "/guides/create-a-plugin/",
                        },
                    ],
                },
            ],
        }),
        react(),
    ],
});
