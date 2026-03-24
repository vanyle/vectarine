// @ts-check
import react from "@astrojs/react";
import starlight from "@astrojs/starlight";
import { defineConfig } from "astro/config";

// https://astro.build/config
export default defineConfig({
    srcDir: "./src",
    build: {
        format: "file",
    },
    base: "/",
    integrations: [
        starlight({
            title: "Vectarine",
            favicon: "./public/vectarine.png",
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
            ],
            sidebar: [
                {
                    label: "Welcome",
                    link: "/",
                },
                {
                    label: "Guides",
                    items: [
                        {
                            label: "Create your first game",
                            link: "/guides/getting-started/",
                        },
                        {
                            label: "Optimizing your game",
                            link: "/guides/use-fastlists/",
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
