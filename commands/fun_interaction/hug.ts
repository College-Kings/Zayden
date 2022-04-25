import Discord from "discord.js";


module.exports = {
    commands: ["hug"],
    expectedArgs: "<user>",
    maxArgs: 1,
    callback: (message: Discord.Message, args: string[], text: string) => {
        let member: Discord.GuildMember | undefined;
        if (message.mentions.members) { member = message.mentions.members.first(); }
        if (!member) { member = message.member as Discord.GuildMember }

        const imageConfig = require("../../configs/image_config.json")
        let arrayId = "Global";
        if (message.author.id in imageConfig.huggingImgs) { arrayId = message.author.id }

        const imgId = Math.floor(Math.random() * imageConfig.huggingImgs[arrayId].length)

        const embed = new Discord.MessageEmbed()
            .setTitle(`Sending hugs to ${member.displayName}`)
            .setImage(imageConfig.huggingImgs[arrayId][imgId])
            .setColor("#FFC0CB")

        message.channel.send({embeds: [embed]})
    },
}