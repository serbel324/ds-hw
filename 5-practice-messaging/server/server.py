from flask import Flask

from config import IMAGES_ENDPOINT, DATA_DIR


def create_app() -> Flask:
    """
    Create flask application
    """
    app = Flask(__name__)

    @app.route(IMAGES_ENDPOINT, methods=['POST'])
    def add_image():
        pass

    @app.route(IMAGES_ENDPOINT, methods=['GET'])
    def get_image_ids():
        pass

    @app.route(f'{IMAGES_ENDPOINT}/<string:image_id>', methods=['GET'])
    def get_processing_result(image_id):
        pass

    return app


app = create_app()

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
