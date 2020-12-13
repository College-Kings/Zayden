const Discord = require("discord.js")
const client = new Discord.Client()

const config = require("./config.json")
const command = require("./command") 

client.on("ready", () => {
    console.log("College King's Bot is Running")

    command(client, "ping", (message) => {
        message.channel.send("Pong!")
    })
})

// let team1 = message.guild.roles.get(config.team1);
// let team2 = message.guild.roles.get(config.team2);

client.on('guildMemberAdd', member => {
    if (Math.floor(Math.random() * 2) == 0) {
        member.addRole(team1).catch(console.error);
    } else {
        member.addRole(team2).catch(console.error);
    }
});



client.login(config.token)
