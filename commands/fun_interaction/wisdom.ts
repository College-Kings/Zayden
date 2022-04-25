import Discord from "discord.js";

module.exports = {
    commands: ["wisdomoftheday", "wisdom", "w"],
    callback: (message: Discord.Message) => {
        const wisdomImages = require("../../configs/image_config.json").wisdomImages

        // Returns 0 - 365
        const now = new Date();
        const start = new Date(now.getFullYear(), 0, 0);
        const oneDay = 1000 * 60 * 60 * 24;
        const imageIndex = Math.floor((now.valueOf() - start.valueOf()) / oneDay)

        // Check if index is within bounds of the global images
        if (imageIndex < wisdomImages.length) {
            const embed = new Discord.MessageEmbed()
                .setTitle("Today's Wisdom")
                .setImage(wisdomImages[imageIndex])

            message.channel.send({embeds: [embed]})
        } else {
            const request = require("request")

            request("https://zenquotes.io/api/today", {json: true}, async (err: any, res: any, body: any) => {
                if (err) {
                    return console.log(err);
                }
                body = body[0]
                const messageContent = `> ${body.q}\n${body.a}\n*(ZenQuotes API)*`
                wisdomImages.push(messageContent)
                await message.channel.send(messageContent)
            });

        }

    },
}