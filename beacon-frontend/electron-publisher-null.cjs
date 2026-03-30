// electron-publisher-null.cjs
class NullPublisher {
    constructor(context) {
        this.context = context;
    }

    async upload(task) {
        console.log(`Fake uploading: ${task.file}`);
        // Do nothing here
        return;
    }
}

module.exports = NullPublisher;