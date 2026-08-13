import { docsLoader } from "@astrojs/starlight/loaders";
import { docsSchema } from "@astrojs/starlight/schema";
import { defineCollection } from "astro:content";
import { blogSchema } from "starlight-blog/schema";

export const collections = {
    docs: defineCollection({
        loader: docsLoader(),
        // The schema is the header of the .mdx file
        schema: docsSchema({
            extend: (context) => blogSchema(context),
        }),
    }),
};
