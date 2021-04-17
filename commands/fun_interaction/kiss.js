const Discord = require("discord.js")
const imgConfig = require("../../configs/imgConfig.json")

module.exports = {
    commands: ["kiss"],
    expectedArgs: "<user>",
    maxArgs: 1,
    callback: (message, arguments, text) => {
        let member;
        try { member = message.mentions.members.first().user.username }
        catch (error) { member = message.author.username }

        let arrayId = "Global";
        if (message.author.id in imgConfig.kissingImgs) { arrayId = message.author.id }
        try {
            if (message.mentions.members.first().user.id in imgConfig.kissingImgs) { arrayId = message.mentions.members.first().user.id }
        }
        catch (error) { arrayId = "Global" }


        const imgId = Math.floor(Math.random() * imgConfig.kissingImgs[arrayId].length)

        const embed = new Discord.MessageEmbed()
            .setTitle(`${message.author.username} kisses ${member}`)
            .setImage(imgConfig.kissingImgs[arrayId][imgId])
            .setColor("FFC0CB")

        message.channel.send(embed)
    },
}
