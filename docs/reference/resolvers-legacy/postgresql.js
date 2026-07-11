/**
 * PostgreSQL resolver — maps PG JDBC raw physical types to aqua logical DataType.
 */

module.exports = {
  type: 'postgresql',

  resolve: function (column) {
    var raw = column.rawType.toUpperCase();
    var length = column.length;
    var precision = column.precision;
    var scale = column.scale;

    // VARCHAR / CHARACTER VARYING / CHAR → VARCHAR(n)
    if (raw === 'VARCHAR' || raw === 'CHARACTER VARYING' || raw === 'CHAR' || raw === 'CHARACTER') {
      return { dataType: 'VARCHAR', length: length > 0 ? length : undefined };
    }

    // TEXT → CLOB
    if (raw === 'TEXT') {
      return { dataType: 'CLOB' };
    }

    // SMALLINT / INT2 → TINYINT
    if (raw === 'SMALLINT' || raw === 'INT2') {
      return { dataType: 'TINYINT' };
    }

    // INTEGER / INT / INT4 → INT
    if (raw === 'INTEGER' || raw === 'INT' || raw === 'INT4') {
      return { dataType: 'INT' };
    }

    // BIGINT / INT8 → LONG
    if (raw === 'BIGINT' || raw === 'INT8') {
      return { dataType: 'LONG' };
    }

    // NUMERIC → DECIMAL(p,s)
    if (raw === 'NUMERIC') {
      var p = precision != null && precision > 0 ? precision : undefined;
      var s = scale != null && scale > 0 ? scale : undefined;
      return { dataType: 'DECIMAL', precision: p, scale: s };
    }

    // DATE → DATE
    if (raw === 'DATE') {
      return { dataType: 'DATE' };
    }

    // TIMESTAMP / TIMESTAMP WITHOUT TIME ZONE → DATETIME
    if (raw === 'TIMESTAMP' || raw === 'TIMESTAMP WITHOUT TIME ZONE' || raw.indexOf('TIMESTAMP') === 0) {
      return { dataType: 'DATETIME' };
    }

    // BYTEA → BLOB
    if (raw === 'BYTEA') {
      return { dataType: 'BLOB' };
    }

    return null;
  }
};
