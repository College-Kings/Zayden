const Discord = require("discord.js")
const config = require("../../serverConfigs/745662812335898806.json")

module.exports = {
    commands: ["suggest", "suggestion"],
    expectedArgs: "<text>",
    permissionError: "",
    minArgs: 1,
    callback: (message, arguments, text) => {
        const embed = new Discord.MessageEmbed()
            .setTitle(`From: ${message.author.username}`)
            .setDescription(text)

        let channel = message.guild.channels.cache.get(config.suggestionChannel)
        channel.send(embed).then(function(message) {
            message.react("👍")
            message.react("👎")
        })
    },
}
