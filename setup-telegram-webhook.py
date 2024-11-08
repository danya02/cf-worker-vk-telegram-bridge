import requests

api_key = input("Telegram API token: ")

def perform_tg_api(method, **args):
    response = requests.post(f'https://api.telegram.org/bot{api_key}/{method}', json=args)
    response.raise_for_status()
    return response.json()

data = perform_tg_api("getWebhookInfo")

if data['result']['url']:
    print(f"Webhook URL: {data['result']['url']}")
else:
    print("No webhook URL found")

url = input("New Webhook URL (empty to remove): ")

if url:
    perform_tg_api("setWebhook", url=url)
    print("Webhook URL updated")
else:
    perform_tg_api("deleteWebhook")
    print("Webhook URL removed")