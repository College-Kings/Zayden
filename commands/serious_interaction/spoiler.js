module.exports = {
    commands: ["spoiler", "spoilers"],
    callback: (message, arguments, text) => {
        message.channel.send(`Please keep all conversations about the new update (v0.9) to <#770621445637799946>
If you have any bugs or questions please post them in <#747428614500384788> and <#747426391263543316>`)
    },
}