const Discord = require("discord.js")
const config = require("./config.json")

const client = new Discord.Client()

client.on("ready", () => {
    console.log("College King's Bot is Running")
})

client.login(config.token)
