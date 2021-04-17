const blacklist = require("../../blacklist");

module.exports = {
    commands: ["botban", "addbotban"],
    expectedArgs: "<user>",
    minArgs: 1,
    callback: (message, arguments, text) => {
        const member = message.mentions.members.first()

        if (!member) {
            message.reply("Please mention a valid user.")
            return
        }

        if (blacklist.isProtectedUser(member.user.id)) {
            message.reply("Nice try you can't botban that user <:pepepointedlaugh:788514455477813320>");
        } else {
            if (blacklist.isBlacklisted(member.user.id)) {
                message.reply("User is already blacklisted!");
            } else {
                blacklist.blacklist(member.user.id);
                message.reply("You have succesfully blacklisted the user!");
            }
        }
    },
    requiredRoles: ["Security"],
}