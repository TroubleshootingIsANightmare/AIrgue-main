'use server';

export default async function get_aires(instance: string) {
    let res = await fetch(`http://127.0.0.1:8080/${instance}`).then(response => response.text());

    console.log(res);

    return res;
}